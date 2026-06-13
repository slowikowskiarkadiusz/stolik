#![no_std]
#![no_main]
use core::prelude::v1::*;

use core::ops::Index;

use common::engine::{
    hash_map::HashMap,
    input::{
        gesture::Gestures,
        input::Input,
        key::{KEYS_LENGTH, Key, KeyState},
    },
};

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use embedded_hal::i2c::I2c;
use esp_hal::{
    Blocking,
    gpio::{AnyPin, Input as GpioInput, InputConfig, Pin, Pull},
    peripherals::{I2C0, Peripherals},
};
use esp_println::{print, println};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum IoPin {
    Gpio1 = 1,
    Gpio18 = 18,
    Gpio21 = 21,
    Gpio35 = 35,
    Gpio36 = 36,
    Gpio37 = 37,
    Gpio38 = 38,
    Gpio39 = 39,
    Gpio40 = 40,
    Gpio41 = 41,
    Gpio42 = 42,
    Gpio47 = 47,
    Gpio48 = 48,
    ExpA0 = 90,
    ExpA1 = 91,
    ExpA2 = 92,
    ExpA3 = 93,
    ExpA4 = 94,
    ExpA5 = 95,
    ExpA6 = 96,
    ExpA7 = 97,
}

const PINS_IN_USE_LENGTH: usize = 18;

static PINS_IN_USE: [IoPin; PINS_IN_USE_LENGTH] = [
    IoPin::Gpio18,
    IoPin::Gpio21,
    IoPin::Gpio35,
    IoPin::Gpio36,
    IoPin::Gpio37,
    IoPin::Gpio38,
    IoPin::Gpio39,
    IoPin::Gpio40,
    IoPin::Gpio47,
    IoPin::Gpio48,
    IoPin::ExpA0,
    IoPin::ExpA1,
    IoPin::ExpA2,
    IoPin::ExpA3,
    IoPin::ExpA4,
    IoPin::ExpA5,
    IoPin::ExpA6,
    IoPin::ExpA7,
];

pub struct Esp32Input<'a> {
    gestures: Gestures,
    gpio_buttons: HashMap<IoPin, GpioInput<'a>>,
    last_level: [bool; PINS_IN_USE_LENGTH],
    keys_down: Vec<IoPin>,
    keys_up: Vec<IoPin>,
    keys_press: Vec<IoPin>,
    // i2c: esp_hal::i2c::master::I2c<'a, esp_hal::Blocking>,
    // expander_present: bool,
    // consecutive_i2c_errors: u8,
}

pub struct Esp32InputPinSetup<'a> {
    pub gpio1: AnyPin<'a>,
    pub gpio18: AnyPin<'a>,
    pub gpio21: AnyPin<'a>,
    pub gpio35: AnyPin<'a>,
    pub gpio36: AnyPin<'a>,
    pub gpio37: AnyPin<'a>,
    pub gpio38: AnyPin<'a>,
    pub gpio39: AnyPin<'a>,
    pub gpio40: AnyPin<'a>,
    pub gpio41: AnyPin<'a>,
    pub gpio42: AnyPin<'a>,
    pub gpio47: AnyPin<'a>,
    pub gpio48: AnyPin<'a>,
}

pub struct Esp32ExpanderPinSetup<'a> {
    pub i2c0: I2C0<'a>,
    pub gpio19: AnyPin<'a>,
    pub gpio20: AnyPin<'a>,
}

impl<'a> Esp32Input<'a> {
    pub fn new(setup: Esp32InputPinSetup<'a>) -> Self {
        let config = InputConfig::default().with_pull(Pull::Up);

        let mut gpio_buttons = HashMap::<IoPin, GpioInput<'a>>::new();
        gpio_buttons.insert(IoPin::Gpio1, GpioInput::new(setup.gpio1, config));
        gpio_buttons.insert(IoPin::Gpio18, GpioInput::new(setup.gpio18, config));
        gpio_buttons.insert(IoPin::Gpio21, GpioInput::new(setup.gpio21, config));
        gpio_buttons.insert(IoPin::Gpio35, GpioInput::new(setup.gpio35, config));
        gpio_buttons.insert(IoPin::Gpio36, GpioInput::new(setup.gpio36, config));
        gpio_buttons.insert(IoPin::Gpio37, GpioInput::new(setup.gpio37, config));
        gpio_buttons.insert(IoPin::Gpio38, GpioInput::new(setup.gpio38, config));
        gpio_buttons.insert(IoPin::Gpio39, GpioInput::new(setup.gpio39, config));
        gpio_buttons.insert(IoPin::Gpio40, GpioInput::new(setup.gpio40, config));
        gpio_buttons.insert(IoPin::Gpio41, GpioInput::new(setup.gpio41, config));
        gpio_buttons.insert(IoPin::Gpio42, GpioInput::new(setup.gpio42, config));
        gpio_buttons.insert(IoPin::Gpio47, GpioInput::new(setup.gpio47, config));
        gpio_buttons.insert(IoPin::Gpio48, GpioInput::new(setup.gpio48, config));

        Self {
            gestures: Gestures::new(),
            // i2c,
            gpio_buttons,
            last_level: [false; PINS_IN_USE_LENGTH],
            keys_down: Vec::new(),
            keys_up: Vec::new(),
            keys_press: Vec::new(),
            // expander_present,
            // consecutive_i2c_errors: 0,
        }
    }

    fn is_pin_down(&mut self, expander_state: u8, pin: IoPin) -> bool {
        if (pin.clone() as u8) < 90 {
            self.gpio_buttons[&pin].is_low()
        } else {
            (expander_state & (1 << (pin as u8 - 90))) == 0
        }
    }

    fn is_key(&self, key: Option<Key>, key_state: KeyState) -> bool {
        if key == None {
            return match key_state {
                KeyState::Down => !self.keys_down.is_empty(),
                KeyState::Up => !self.keys_up.is_empty(),
                KeyState::Press => !self.keys_press.is_empty(),
            };
        };

        let pins = Esp32Input::map_key(key.unwrap());
        match key_state {
            KeyState::Down => pins.iter().any(|p| self.keys_down.contains(p)),
            KeyState::Up => pins.iter().any(|p| self.keys_up.contains(p)),
            KeyState::Press => pins.iter().any(|p| self.keys_press.contains(p)),
        }
    }

    fn map_key(key: Key) -> Vec<IoPin> {
        match key {
            Key::Start => vec![IoPin::Gpio1],
            Key::P1Down => vec![IoPin::Gpio39],
            Key::P1Up => vec![IoPin::Gpio36],
            Key::P1Left => vec![IoPin::Gpio35],
            Key::P1Right => vec![IoPin::Gpio37],
            Key::P1AnyDirection => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P1Down));
                keys.append(&mut Esp32Input::map_key(Key::P1Up));
                keys.append(&mut Esp32Input::map_key(Key::P1Left));
                keys.append(&mut Esp32Input::map_key(Key::P1Right));
                keys
            }
            Key::P1Blue => vec![IoPin::Gpio47],
            Key::P1Green => vec![IoPin::Gpio48],
            Key::P1Any => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P1AnyDirection));
                keys.append(&mut Esp32Input::map_key(Key::P1Blue));
                keys.append(&mut Esp32Input::map_key(Key::P1Green));
                keys
            }
            Key::P2Down => vec![IoPin::ExpA1],
            Key::P2Up => vec![IoPin::ExpA3],
            Key::P2Left => vec![IoPin::ExpA0],
            Key::P2Right => vec![IoPin::ExpA2],
            Key::P2AnyDirection => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P2Down));
                keys.append(&mut Esp32Input::map_key(Key::P2Up));
                keys.append(&mut Esp32Input::map_key(Key::P2Left));
                keys.append(&mut Esp32Input::map_key(Key::P2Right));
                keys
            }
            Key::P2Blue => vec![IoPin::ExpA7],
            Key::P2Green => vec![IoPin::ExpA6],
            Key::P2Any => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P2AnyDirection));
                keys.append(&mut Esp32Input::map_key(Key::P2Blue));
                keys.append(&mut Esp32Input::map_key(Key::P2Green));
                keys
            }
        }
    }
}

impl<'a> Input for Esp32Input<'a> {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let expander_data = EXPANDER_DATA.try_take().unwrap_or(0xFF);
        // let expander_data = 0;

        for i in 0..PINS_IN_USE.len() {
            let pin = &PINS_IN_USE[i];
            let is_down = self.is_pin_down(expander_data, pin.clone());

            if is_down != self.last_level[i] {
                if is_down && !self.keys_press.contains(pin) {
                    self.keys_down.push(pin.clone());
                    self.keys_press.push(pin.clone());
                }

                if !is_down {
                    self.keys_up.push(pin.clone());
                    let position = self.keys_press.iter().position(|f| f == pin).unwrap();
                    self.keys_press.remove(position);
                }
            }

            self.last_level[i] = is_down;
        }

        self.gestures.tick(self.get_snapshot(), delta_time);
    }

    fn late_update(&mut self, _: f32) {
        self.keys_down.clear();
        self.keys_up.clear();
        self.gestures.late_tick();
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Down)
    }

    fn is_any_key_down(&self) -> bool {
        self.is_key(None, KeyState::Down)
    }

    fn is_key_up(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Up)
    }

    fn is_any_key_up(&self) -> bool {
        self.is_key(None, KeyState::Up)
    }

    fn is_key_press(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Press)
    }

    fn is_any_key_press(&self) -> bool {
        self.is_key(None, KeyState::Press)
    }

    fn clear(&mut self) {
        self.keys_down.clear();
        self.keys_up.clear();
        self.keys_press.clear();
    }
}

use alloc::collections::btree_map::BTreeMap;
use embedded_hal_compat::Reverse;
use embedded_hal_compat::ReverseCompat;
static EXPANDER_DATA: Signal<CriticalSectionRawMutex, u8> = Signal::new();
use pcf857x::{Pcf8574, PinFlag, SlaveAddr};

#[task]
pub async fn read_expander_data(setup: Esp32ExpanderPinSetup<'static>) {
    let i2c_config = esp_hal::i2c::master::Config::default().with_software_timeout(esp_hal::i2c::master::SoftwareTimeout::Transaction(
        esp_hal::time::Duration::from_millis(5),
    ));
    let mut i2c = esp_hal::i2c::master::I2c::new(setup.i2c0, i2c_config)
        .unwrap()
        .with_sda(setup.gpio19)
        .with_scl(setup.gpio20);

    let i2c_compat = i2c.reverse();
    let mut expander = Pcf8574::new(i2c_compat, SlaveAddr::default());

    let mask =
        PinFlag::P0
        | PinFlag::P1
        | PinFlag::P2
        | PinFlag::P3
        | PinFlag::P4
        | PinFlag::P5
        | PinFlag::P6
        | PinFlag::P7;
        // |
        // PinFlag::P10
        // | PinFlag::P11
        // | PinFlag::P12
        // | PinFlag::P13
        // | PinFlag::P14
        // | PinFlag::P15
        // | PinFlag::P16
        // | PinFlag::P17;

    let mut i = 0;

    loop {
        if let Ok(inputs) = expander.get(&mask) {
            EXPANDER_DATA.signal(inputs);
            // println!("hej {}", inputs);
        } else {
            i += 1;
            i %= 100;
            println!("Halo {}", i);
        }

        Timer::after_millis(10).await;
    }
}

fn read(addr: u8, i2c: &mut esp_hal::i2c::master::I2c<'_, esp_hal::Blocking>) -> u8 {
    return 0xff;
    let expander_present = i2c.write(addr, &[0x00, 0xFF]).is_ok() && i2c.write(addr, &[0x01, 0xFF]).is_ok();
    if !expander_present {
        return 0;
    }
    let mut buf = [0u8; 1];
    if i2c.write_read(addr, &[0x12], &mut buf).is_err() {
        return 0;
    }
    buf[0]
}
