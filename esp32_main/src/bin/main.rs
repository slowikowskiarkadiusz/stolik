//! Embassy "async" example of an ESP32-S3 driving 4 64x32 HUB75 displays
//! in a 2x2 arrangement using the LCD_CAM peripheral.
//!
//! This example draws a simple gradient on the displays and shows the refresh
//! rate and render rate plus a simple counter.
//!
//! Panel arrangement (front view):
//! ```text
//! ┌─────────────────────────────────┐
//! │  Panel 1        │  Panel 0      │
//! │  (64x32)        <  (64x32)      │
//! │  Top-Left       <  Top-Right    │
//! │                 │               │
//! ├────────vv───────┼───────────────┤
//! │  Panel 2        │  Panel 3      │
//! │  (64x32)        >  (64x32)      │
//! │  Bottom-Left    >  Bottom-Right │
//! │                 │               │
//! └─────────────────────────────────┘
//! Total display area: 128x64 pixels
//! ```
//!
//! ⚠️ **IMPORTANT:** Panels 2 and 3 (second row) need to be mounted upside down
//! to maintain the correct data flow direction indicated by the arrows.
//!
//! Folowing pins are used:
//! - R1  => GPIO38
//! - G1  => GPIO42
//! - B1  => GPIO48
//! - R2  => GPIO47
//! - G2  => GPIO2
//! - B2  => GPIO21
//! - A   => GPIO14
//! - B   => GPIO46
//! - C   => GPIO13
//! - D   => GPIO9
//! - E   => GPIO3
//! - OE  => GPIO11
//! - CLK => GPIO12
//! - LAT => GPIO10
//!
//! Note that you most likely need level converters 3.3v to 5v for all HUB75
//! signals
//!
#![no_std]
#![no_main]
#![allow(clippy::uninlined_format_args)]
use core::fmt;
use core::prelude::v1::*;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;
use esp_alloc;

use alloc::boxed::Box;
use common::engine::color::Color;
use common::engine::color_matrix::ColorMatrix;
use common::engine::engine::{Engine, SCREEN_SIZE};
#[cfg(feature = "defmt")]
use defmt::info;
#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_executor::task;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Duration;
use embassy_time::Instant;
use embassy_time::Timer;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::FONT_5X7;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::text::Alignment;
use embedded_graphics::text::Text;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::Pin;
use esp_hal::interrupt::Priority;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::peripherals::LCD_CAM;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
extern crate alloc;
use alloc::sync::Arc;
use esp_hub75::Color as Esp32Color;
use esp_hub75::Hub75;
use esp_hub75::Hub75Pins16;
use esp_hub75::framebuffer::FrameBufferOperations;
use esp_hub75::framebuffer::compute_frame_count;
use esp_hub75::framebuffer::compute_rows;
use esp_hub75::framebuffer::plain::DmaFrameBuffer;
use esp_hub75::framebuffer::tiling::ChainTopRightDown;
use esp_hub75::framebuffer::tiling::TiledFrameBuffer;
use esp_hub75::framebuffer::tiling::compute_tiled_cols;
use esp_println::println;
use esp_rtos::embassy::Executor;
use esp_rtos::embassy::InterruptExecutor;
use esp32_main::esp32_input::Esp32Input;
use esp32_main::esp32_input::Esp32InputPinSetup;
use esp32_main::esp32_threading_provider::Esp32Thread;
use spin::Mutex;
// use esp_rtos::embassy::InterruptExecutor;
use heapless::String;
#[cfg(feature = "log")]
use log::info;

const SCREEN_PIXELS: usize = 64 * 64;

/// Static frame buffer — no heap allocation for the rendered frame.
struct SharedFrame {
    data: [Color; SCREEN_PIXELS],
    valid: bool,
}

impl SharedFrame {
    const fn new() -> Self {
        Self {
            data: [Color::none(); SCREEN_PIXELS],
            valid: false,
        }
    }
}

static SHARED: Mutex<SharedFrame> = Mutex::new(SharedFrame::new());

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

// These are some values that can be tweaked to experiment with the display

const TRANSFER_SPEED: Rate = Rate::from_mhz(10);
const BITS: u8 = 3;

// Panel layout settings
const TILED_COLS: usize = 1;
const TILED_ROWS: usize = 2;
const PANEL_ROWS: usize = 32;
const PANEL_COLS: usize = 64;
const FB_COLS: usize = compute_tiled_cols(PANEL_COLS, TILED_ROWS, TILED_COLS);
const NROWS: usize = compute_rows(PANEL_ROWS);
const FRAME_COUNT: usize = compute_frame_count(BITS);
const VIRTUAL_ROWS: usize = TILED_ROWS * PANEL_ROWS;
const VIRTUAL_COLS: usize = TILED_COLS * PANEL_COLS;

static REFRESH_RATE: AtomicU32 = AtomicU32::new(0);
static RENDER_RATE: AtomicU32 = AtomicU32::new(0);
static SIMPLE_COUNTER: AtomicU32 = AtomicU32::new(0);

type FBType = DmaFrameBuffer<PANEL_ROWS, FB_COLS, NROWS, BITS, FRAME_COUNT>;
type TiledFBType = TiledFrameBuffer<
    FBType,
    ChainTopRightDown<PANEL_ROWS, PANEL_COLS, TILED_ROWS, TILED_COLS>,
    PANEL_ROWS,
    PANEL_COLS,
    NROWS,
    BITS,
    FRAME_COUNT,
    TILED_ROWS,
    TILED_COLS,
    FB_COLS,
>;

type FrameBufferExchange = Signal<CriticalSectionRawMutex, &'static mut TiledFBType>;

pub struct Hub75Peripherals<'d> {
    pub lcd_cam: LCD_CAM<'d>,
    pub dma_channel: esp_hal::peripherals::DMA_CH0<'d>,
    pub pins: Hub75Pins16<'d>,
}

#[task]
async fn hub75_task(
    peripherals: Hub75Peripherals<'static>,
    rx: &'static FrameBufferExchange,
    tx: &'static FrameBufferExchange,
    fb: &'static mut TiledFBType,
) {
    // info!("hub75_task: starting!");

    let tx_descriptors = esp_hub75::hub75_dma_descriptors!(FBType);
    //println!("hub75: descriptors allocated");

    let mut hub75 = Hub75::new_async(
        peripherals.lcd_cam,
        peripherals.pins,
        peripherals.dma_channel,
        tx_descriptors,
        TRANSFER_SPEED,
    )
    .expect("failed to create Hub75!");
    //println!("hub75: hub75 created");

    let mut count = 0u32;
    let mut start = Instant::now();

    let mut fb = fb;
    let mut iter = 0u32;

    loop {
        iter += 1;
        if iter <= 5 || iter % 100 == 0 {
            //println!("hub75: loop iter {}", iter);
        }

        // if there is a new buffer available, get it and send the old one
        if rx.signaled() {
            let new_fb = rx.wait().await;
            tx.signal(fb);
            fb = new_fb;
        }

        let mut xfer = hub75.render(fb).map_err(|(e, _hub75)| e).expect("failed to start render!");
        if iter <= 5 {
            //println!("hub75: render started iter {}", iter);
        }
        xfer.wait_for_done().await.expect("rendering wait_for_done failed!");
        if iter <= 5 {
            //println!("hub75: render done iter {}", iter);
        }
        let (result, new_hub75) = xfer.wait();
        hub75 = new_hub75;
        result.expect("transfer failed");

        count += 1;
        const FPS_INTERVAL: Duration = Duration::from_secs(1);
        if start.elapsed() > FPS_INTERVAL {
            REFRESH_RATE.store(count, Ordering::Relaxed);
            count = 0;
            start = Instant::now();
        }
    }
}

#[task]
async fn display_task(rx: &'static FrameBufferExchange, tx: &'static FrameBufferExchange, mut fb: &'static mut TiledFBType) {
    let mut count = 0u32;
    let mut start = Instant::now();
    let mut diag_count = 0u32;

    loop {
        fb.erase();

        // Read frame data directly from static buffer
        let mut nonzero = 0u32;
        {
            let s = SHARED.lock();
            if s.valid {
                for y in 0..64u8 {
                    for x in 0..64u8 {
                        let (mut sample_x, mut sample_y) = if y < 32 { (63 - x, 31 - y) } else { (x, y) };
                        (sample_x, sample_y) = (63 - sample_x, 63 - sample_y);
                        let color = &s.data[sample_y as usize * 64 + sample_x as usize];
                        if color.a > 0 {
                            nonzero += 1;
                            let esp32_color = Esp32Color::new(
                                (color.r as u16 * color.a as u16 / 255) as u8,
                                (color.g as u16 * color.a as u16 / 255) as u8,
                                (color.b as u16 * color.a as u16 / 255) as u8,
                            );
                            fb.set_pixel(Point::new(x as i32, y as i32), esp32_color);
                        }
                    }
                }
            }
        }

        // Corner markers — help identify which part of the virtual screen is visible:
        // RED 4x4 at (0,0), GREEN at (60,0), BLUE at (0,60), YELLOW at (60,60), CYAN at (30,30)
        // for dy in 0i32..4 {
        //     for dx in 0i32..4 {
        //         fb.set_pixel(Point::new(dx,    dy),    Esp32Color::RED);
        //         fb.set_pixel(Point::new(60+dx, dy),    Esp32Color::GREEN);
        //         fb.set_pixel(Point::new(dx,    60+dy), Esp32Color::BLUE);
        //         fb.set_pixel(Point::new(60+dx, 60+dy), Esp32Color::YELLOW);
        //         fb.set_pixel(Point::new(30+dx, 30+dy), Esp32Color::CYAN);
        //     }
        // }

        diag_count += 1;
        if diag_count <= 5 || diag_count % 100 == 0 {
            //println!("display: iter={} nonzero_pixels={}", diag_count, nonzero);
        }

        // send the frame buffer to be rendered
        tx.signal(fb);

        // get the next frame buffer
        fb = rx.wait().await;

        // count up the rate we are rendering full buffer
        count += 1;
        const FPS_INTERVAL: Duration = Duration::from_secs(1);
        if start.elapsed() > FPS_INTERVAL {
            RENDER_RATE.store(count, Ordering::Relaxed);
            count = 0;
            start = Instant::now();
        }
    }
}

// fn draw(x: u8, y: u8, color: Color) {
//     let esp32_color = Esp32Color::new((color.r as f32 * (color.a as f32 / 255.0)) as u8, (color.g as f32 * (color.a as f32 / 255.0)) as u8, (color.b as f32 * (color.a as f32 / 255.0)) as u8);

//   if (y < 32) {
//     dma_display->drawPixel(x, y, c);
//   } else {
//     dma_display->drawPixel(x + 64, y - 32, c);
//   }
// }

unsafe extern "C" {
    static _stack_end_cpu0: u32;
    static _stack_start_cpu0: u32;
}

#[esp_rtos::main]
async fn main(_s: embassy_executor::Spawner) {
    esp_alloc::heap_allocator!(64 * 1024);
    #[cfg(feature = "log")]
    esp_println::logger::init_logger_from_env();
    // info!("Main starting!");
    // info!("main: stack size:  {}", unsafe {
    // core::ptr::addr_of!(_stack_start_cpu0).offset_from(core::ptr::addr_of!(_stack_end_cpu0))
    // });
    // info!("VIRTUAL_ROWS: {}", VIRTUAL_ROWS);
    // info!("VIRTUAL_COLS: {}", VIRTUAL_COLS);
    // info!("BITS: {}", BITS);
    // info!("FRAME_COUNT: {}", FRAME_COUNT);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let sw_ints = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let software_interrupt = sw_ints.software_interrupt2;

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_rtos::start(timer0.alarm0);

    // info!("Embassy initialized!");

    let input_pin_setup = Esp32InputPinSetup {
        gpio1: peripherals.GPIO1.degrade(),
        // gpio17: peripherals.GPIO17.degrade(),
        gpio18: peripherals.GPIO18.degrade(),
        gpio21: peripherals.GPIO21.degrade(),
        gpio35: peripherals.GPIO35.degrade(),
        gpio36: peripherals.GPIO36.degrade(),
        gpio37: peripherals.GPIO37.degrade(),
        gpio38: peripherals.GPIO38.degrade(),
        gpio39: peripherals.GPIO39.degrade(),
        gpio40: peripherals.GPIO40.degrade(),
        gpio41: peripherals.GPIO41.degrade(),
        gpio42: peripherals.GPIO42.degrade(),
        gpio47: peripherals.GPIO47.degrade(),
        gpio48: peripherals.GPIO48.degrade(),
    };

    let expander_pin_setup = esp32_main::esp32_input::Esp32ExpanderPinSetup {
        i2c0: peripherals.I2C0,
        gpio19: peripherals.GPIO19.degrade(),
        gpio20: peripherals.GPIO20.degrade(),
    };

    let pins = Hub75Pins16 {
        red1: peripherals.GPIO4.degrade(),
        grn1: peripherals.GPIO5.degrade(),
        blu1: peripherals.GPIO7.degrade(),
        red2: peripherals.GPIO8.degrade(),
        grn2: peripherals.GPIO9.degrade(),
        blu2: peripherals.GPIO10.degrade(),
        addr0: peripherals.GPIO11.degrade(),
        addr1: peripherals.GPIO12.degrade(),
        addr2: peripherals.GPIO13.degrade(),
        addr3: peripherals.GPIO14.degrade(),
        addr4: peripherals.GPIO3.degrade(),
        blank: peripherals.GPIO17.degrade(),
        clock: peripherals.GPIO15.degrade(),
        latch: peripherals.GPIO16.degrade(),
    };

    let hub75_per = Hub75Peripherals {
        dma_channel: peripherals.DMA_CH0,
        lcd_cam: peripherals.LCD_CAM,
        pins,
    };

    // info!("init framebuffer exchange");
    static TX: FrameBufferExchange = FrameBufferExchange::new();
    static RX: FrameBufferExchange = FrameBufferExchange::new();

    let cpu1_fnctn = {
        move || {
            // Framebuffers are created here on CPU1 (128 KB stack) to avoid
            // overflowing CPU0's stack with the large TiledFrameBuffer temporary.
            let fb0 = mk_static!(TiledFBType, TiledFrameBuffer::new());
            let fb1 = mk_static!(TiledFBType, TiledFrameBuffer::new());

            let hp_executor = mk_static!(InterruptExecutor<2>, InterruptExecutor::new(software_interrupt));
            let high_pri_spawner = hp_executor.start(Priority::Priority3);

            // hub75 runs as high priority task
            high_pri_spawner.spawn(hub75_task(hub75_per, &RX, &TX, fb1)).ok();

            let lp_executor = mk_static!(Executor, Executor::new());
            // display task runs as low priority task
            lp_executor.run(|spawner: Spawner| {
                spawner.spawn(run_engine(input_pin_setup)).ok();
                spawner.spawn(esp32_main::esp32_input::read_expander_data(expander_pin_setup)).ok();
                spawner.spawn(display_task(&TX, &RX, fb0)).ok();
            });
        }
    };

    use esp_hal::system::Stack;
    const DISPLAY_STACK_SIZE: usize = 131072;
    let app_core_stack = mk_static!(Stack<DISPLAY_STACK_SIZE>, Stack::new());

    esp_rtos::start_second_core(
        peripherals.CPU_CTRL,
        sw_ints.software_interrupt0,
        sw_ints.software_interrupt1,
        app_core_stack,
        cpu1_fnctn,
    );

    loop {
        if SIMPLE_COUNTER.fetch_add(1, Ordering::Relaxed) >= 99999 {
            SIMPLE_COUNTER.store(0, Ordering::Relaxed);
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[task]
async fn run_engine(input_pin_setup: Esp32InputPinSetup<'static>) {
    //println!("engine: creating input");
    let mut engine = Engine::new(Box::new(Esp32Input::new(input_pin_setup)));
    //println!("engine: input ok");
    let on_frame_func: Arc<dyn Fn(&ColorMatrix) + Send + Sync + 'static> = Arc::new(move |mat: &ColorMatrix| {
        let mut s = SHARED.lock();
        s.data.copy_from_slice(&mat.data);
        s.valid = true;
    });

    //println!("engine: ensure_scene");
    engine.ensure_scene();
    //println!("engine: scene ready, entering loop");

    let target_frame = Duration::from_millis(33);
    let mut last = Instant::now();
    let mut frame_count = 0u32;

    loop {
        let frame_start = Instant::now();
        let dt = frame_start.duration_since(last);
        last = frame_start;

        if frame_count < 5 {
            //println!("engine: pre-tick {}", frame_count + 1);
        }
        //println!("engine: pre-tick {}", frame_count + 1);
        engine.tick_frame(dt.as_millis() as f32 / 1000.0, &on_frame_func);
        Timer::after(Duration::from_millis(0)).await;
        //println!("engine: post-tick {}", frame_count + 1);

        frame_count += 1;
        if frame_count <= 10 || frame_count % 100 == 0 {
            //println!("engine: frame {}", frame_count);
        }

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame {
            Timer::after(target_frame - elapsed).await;
        } else {
            Timer::after(Duration::from_millis(0)).await;
        }
    }
}
