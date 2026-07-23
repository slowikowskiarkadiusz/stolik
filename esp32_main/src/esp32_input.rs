use common::engine::{
    hash_map::HashMap,
    input::{
        gesture::Gestures,
        input::Input,
        key::{Key, KeyState},
    },
};

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use core::sync::atomic::{AtomicU8, Ordering};
use embassy_executor::task;
use embassy_time::Timer;
use esp_hal::{
    gpio::{AnyPin, Input as GpioInput, InputConfig, Pull},
    peripherals::I2C0,
};

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
    player: u8,
    gestures: Gestures,
    gpio_buttons: HashMap<IoPin, GpioInput<'a>>,
    last_level: [bool; PINS_IN_USE_LENGTH],
    keys_down: Vec<IoPin>,
    keys_up: Vec<IoPin>,
    keys_press: Vec<IoPin>,
}

pub struct Esp32InputPinSetup<'a> {
    pub gpio1: Option<AnyPin<'a>>,
    pub gpio18: Option<AnyPin<'a>>,
    pub gpio21: Option<AnyPin<'a>>,
    pub gpio35: Option<AnyPin<'a>>,
    pub gpio36: Option<AnyPin<'a>>,
    pub gpio37: Option<AnyPin<'a>>,
    pub gpio38: Option<AnyPin<'a>>,
    pub gpio39: Option<AnyPin<'a>>,
    pub gpio40: Option<AnyPin<'a>>,
    pub gpio41: Option<AnyPin<'a>>,
    pub gpio42: Option<AnyPin<'a>>,
    pub gpio47: Option<AnyPin<'a>>,
    pub gpio48: Option<AnyPin<'a>>,
}

impl<'a> Esp32InputPinSetup<'a> {
    pub fn empty() -> Self {
        Self {
            gpio1: None, gpio18: None, gpio21: None, gpio35: None, gpio36: None,
            gpio37: None, gpio38: None, gpio39: None, gpio40: None, gpio41: None,
            gpio42: None, gpio47: None, gpio48: None,
        }
    }
}

pub struct Esp32ExpanderPinSetup<'a> {
    pub i2c0: I2C0<'a>,
    pub gpio19: AnyPin<'a>,
    pub gpio20: AnyPin<'a>,
}

impl<'a> Esp32Input<'a> {
    pub fn new(player: u8, setup: Esp32InputPinSetup<'a>) -> Self {
        let config = InputConfig::default().with_pull(Pull::Up);

        let mut gpio_buttons = HashMap::<IoPin, GpioInput<'a>>::new();
        macro_rules! insert_pin {
            ($field:expr, $key:expr) => {
                if let Some(pin) = $field {
                    gpio_buttons.insert($key, GpioInput::new(pin, config));
                }
            };
        }
        insert_pin!(setup.gpio1, IoPin::Gpio1);
        insert_pin!(setup.gpio18, IoPin::Gpio18);
        insert_pin!(setup.gpio21, IoPin::Gpio21);
        insert_pin!(setup.gpio35, IoPin::Gpio35);
        insert_pin!(setup.gpio36, IoPin::Gpio36);
        insert_pin!(setup.gpio37, IoPin::Gpio37);
        insert_pin!(setup.gpio38, IoPin::Gpio38);
        insert_pin!(setup.gpio39, IoPin::Gpio39);
        insert_pin!(setup.gpio40, IoPin::Gpio40);
        insert_pin!(setup.gpio41, IoPin::Gpio41);
        insert_pin!(setup.gpio42, IoPin::Gpio42);
        insert_pin!(setup.gpio47, IoPin::Gpio47);
        insert_pin!(setup.gpio48, IoPin::Gpio48);

        Self {
            player,
            gestures: Gestures::new(),
            gpio_buttons,
            last_level: [false; PINS_IN_USE_LENGTH],
            keys_down: Vec::new(),
            keys_up: Vec::new(),
            keys_press: Vec::new(),
        }
    }

    fn is_pin_down(&mut self, expander_state: u8, pin: IoPin) -> bool {
        if (pin.clone() as u8) < 90 {
            self.gpio_buttons.get(&pin).map(|p| p.is_low()).unwrap_or(false)
        } else {
            (expander_state & (1 << (pin.clone() as u8 - 90))) == 0
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

        let pins = self.map_key(key.unwrap());
        match key_state {
            KeyState::Down => pins.iter().any(|p| self.keys_down.contains(p)),
            KeyState::Up => pins.iter().any(|p| self.keys_up.contains(p)),
            KeyState::Press => pins.iter().any(|p| self.keys_press.contains(p)),
        }
    }

    fn map_key(&self, key: Key) -> Vec<IoPin> {
        if self.player == 0 {
        match key {
            Key::Start => vec![IoPin::Gpio40],
            Key::Down => vec![IoPin::Gpio39],
            Key::Up => vec![IoPin::Gpio36],
            Key::Left => vec![IoPin::Gpio35],
            Key::Right => vec![IoPin::Gpio37],
            Key::AnyDirection => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut self.map_key(Key::Down));
                keys.append(&mut self.map_key(Key::Up));
                keys.append(&mut self.map_key(Key::Left));
                keys.append(&mut self.map_key(Key::Right));
                keys
            }
            Key::Blue => vec![IoPin::Gpio47],
            Key::Green => vec![IoPin::Gpio48],
            Key::Any => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut self.map_key(Key::AnyDirection));
                keys.append(&mut self.map_key(Key::Blue));
                keys.append(&mut self.map_key(Key::Green));
                keys
            }
        }
    }
    else {
        match key {
            Key::Start => vec![IoPin::Gpio40],
            Key::Down => vec![IoPin::ExpA1],
            Key::Up => vec![IoPin::ExpA3],
            Key::Left => vec![IoPin::ExpA0],
            Key::Right => vec![IoPin::ExpA2],
            Key::AnyDirection => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut self.map_key(Key::Down));
                keys.append(&mut self.map_key(Key::Up));
                keys.append(&mut self.map_key(Key::Left));
                keys.append(&mut self.map_key(Key::Right));
                keys
            }
            Key::Blue => vec![IoPin::ExpA7],
            Key::Green => vec![IoPin::ExpA6],
            Key::Any => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut self.map_key(Key::AnyDirection));
                keys.append(&mut self.map_key(Key::Blue));
                keys.append(&mut self.map_key(Key::Green));
                keys
            }
        }
    }
    }
}

impl<'a> Input for Esp32Input<'a> {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let expander_data = EXPANDER_DATA.load(Ordering::Relaxed);
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

static EXPANDER_DATA: AtomicU8 = AtomicU8::new(0xFF);

#[task]
pub async fn read_expander_data(setup: Esp32ExpanderPinSetup<'static>) {
    let i2c_config = esp_hal::i2c::master::Config::default()
        .with_software_timeout(esp_hal::i2c::master::SoftwareTimeout::None);
    let mut i2c = esp_hal::i2c::master::I2c::new(setup.i2c0, i2c_config)
        .unwrap()
        .with_sda(setup.gpio19)
        .with_scl(setup.gpio20);

    const ADDR: u8 = 0x20;

    let _ = i2c.write(ADDR, &[0x01, 0xFF]);
    let _ = i2c.write(ADDR, &[0x0D, 0xFF]);

    loop {
        let mut data = [0xFFu8; 2];
        if i2c.write_read(ADDR, &[0x12], &mut data).is_ok() {
            EXPANDER_DATA.store(data[1], Ordering::Relaxed);
        } else {
            let _ = i2c.apply_config(&esp_hal::i2c::master::Config::default()
                .with_software_timeout(esp_hal::i2c::master::SoftwareTimeout::None));
        }
        Timer::after_millis(10).await;
    }
}
