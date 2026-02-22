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

use esp_hal::{
    gpio::{Input as GpioInput, InputConfig, Pin, Pull},
    peripherals::Peripherals,
};

#[derive(Clone, PartialEq, Eq, Hash)]
enum IoPin {
    Gpio1 = 1,
    Gpio17 = 17,
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
    ExpP0 = 90,
    ExpP1 = 91,
    ExpP2 = 92,
    ExpP3 = 93,
    ExpP4 = 94,
    ExpP5 = 95,
    ExpP6 = 96,
    ExpP7 = 97,
}

const PINS_IN_USE_LENGTH: usize = 19;

static PINS_IN_USE: [IoPin; PINS_IN_USE_LENGTH] = [
    IoPin::Gpio17,
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
    IoPin::ExpP0,
    IoPin::ExpP1,
    IoPin::ExpP2,
    IoPin::ExpP3,
    IoPin::ExpP4,
    IoPin::ExpP5,
    IoPin::ExpP6,
    IoPin::ExpP7,
];

pub struct Esp32Input<'a> {
    gestures: Gestures,
    gpio_buttons: HashMap<IoPin, GpioInput<'a>>,
    last_level: [bool; PINS_IN_USE_LENGTH], // true is down so pressed
    keys_down: Vec<IoPin>,
    keys_up: Vec<IoPin>,
    keys_press: Vec<IoPin>,
    i2c: esp_hal::i2c::master::I2c<'a, esp_hal::Blocking>,
}

impl<'a> Esp32Input<'a> {
    pub fn new() -> Self {
        let peripherals = esp_hal::init(esp_hal::Config::default());
        let config = InputConfig::default().with_pull(Pull::Up);

        let mut gpio_buttons = HashMap::<IoPin, GpioInput<'a>>::new();
        gpio_buttons.insert(IoPin::Gpio1, GpioInput::new(peripherals.GPIO1, config));
        gpio_buttons.insert(IoPin::Gpio17, GpioInput::new(peripherals.GPIO17, config));
        gpio_buttons.insert(IoPin::Gpio18, GpioInput::new(peripherals.GPIO18, config));
        gpio_buttons.insert(IoPin::Gpio21, GpioInput::new(peripherals.GPIO21, config));
        gpio_buttons.insert(IoPin::Gpio35, GpioInput::new(peripherals.GPIO35, config));
        gpio_buttons.insert(IoPin::Gpio36, GpioInput::new(peripherals.GPIO36, config));
        gpio_buttons.insert(IoPin::Gpio37, GpioInput::new(peripherals.GPIO37, config));
        gpio_buttons.insert(IoPin::Gpio38, GpioInput::new(peripherals.GPIO38, config));
        gpio_buttons.insert(IoPin::Gpio39, GpioInput::new(peripherals.GPIO39, config));
        gpio_buttons.insert(IoPin::Gpio40, GpioInput::new(peripherals.GPIO40, config));
        gpio_buttons.insert(IoPin::Gpio41, GpioInput::new(peripherals.GPIO41, config));
        gpio_buttons.insert(IoPin::Gpio42, GpioInput::new(peripherals.GPIO42, config));
        gpio_buttons.insert(IoPin::Gpio47, GpioInput::new(peripherals.GPIO47, config));
        gpio_buttons.insert(IoPin::Gpio48, GpioInput::new(peripherals.GPIO48, config));

        let i2c = esp_hal::i2c::master::I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
            .unwrap()
            .with_sda(peripherals.GPIO19)
            .with_scl(peripherals.GPIO20);

        let obj = Self {
            gestures: Gestures::new(),
            i2c,
            gpio_buttons: gpio_buttons,
            last_level: [false; PINS_IN_USE_LENGTH],
            keys_down: Vec::new(),
            keys_up: Vec::new(),
            keys_press: Vec::new(),
        };

        obj
    }

    fn read_expander_data(&mut self) -> u8 {
        let addr: u8 = 0x20;

        self.i2c.write(addr, &[0x00, 0xFF]).unwrap();
        self.i2c.write(addr, &[0x01, 0xFF]).unwrap();

        let mut buf = [0u8; 1];
        self.i2c.write_read(addr, &[0x12], &mut buf).unwrap();
        buf[0]
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

        // let pins_for_key
        false
    }

    fn map_key(key: Key) -> Vec<IoPin> {
        match key {
            Key::Start => {
                return vec![IoPin::Gpio1];
            }
            Key::P1Down => {
                return vec![IoPin::Gpio39];
            }
            Key::P1Up => {
                return vec![IoPin::Gpio36];
            }
            Key::P1Left => {
                return vec![IoPin::Gpio35];
            }
            Key::P1Right => {
                return vec![IoPin::Gpio37];
            }
            Key::P1AnyDirection => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P1Down));
                keys.append(&mut Esp32Input::map_key(Key::P1Up));
                keys.append(&mut Esp32Input::map_key(Key::P1Left));
                keys.append(&mut Esp32Input::map_key(Key::P1Right));
                return keys;
            }
            Key::P1Blue => {
                return vec![IoPin::Gpio47];
            }
            Key::P1Green => {
                return vec![IoPin::Gpio48];
            }
            Key::P1Any => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P1AnyDirection));
                keys.append(&mut Esp32Input::map_key(Key::P1Blue));
                keys.append(&mut Esp32Input::map_key(Key::P1Green));
                return keys;
            }

            Key::P2Down => {
                return vec![IoPin::ExpP0];
            }
            Key::P2Up => {
                return vec![IoPin::ExpP1];
            }
            Key::P2Left => {
                return vec![IoPin::ExpP2];
            }
            Key::P2Right => {
                return vec![IoPin::ExpP3];
            }
            Key::P2AnyDirection => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P2Down));
                keys.append(&mut Esp32Input::map_key(Key::P2Up));
                keys.append(&mut Esp32Input::map_key(Key::P2Left));
                keys.append(&mut Esp32Input::map_key(Key::P2Right));
                return keys;
            }
            Key::P2Blue => {
                return vec![IoPin::ExpP4];
            }
            Key::P2Green => {
                return vec![IoPin::ExpP5];
            }
            Key::P2Any => {
                let mut keys = Vec::<IoPin>::new();
                keys.append(&mut Esp32Input::map_key(Key::P2AnyDirection));
                keys.append(&mut Esp32Input::map_key(Key::P2Blue));
                keys.append(&mut Esp32Input::map_key(Key::P2Green));
                return keys;
            }
        }
    }
}

impl<'a> Input for Esp32Input<'a> {
    fn gestures(&self) -> &Gestures {
        &self.gestures
    }

    fn update(&mut self, delta_time: f32) {
        let expander_data = self.read_expander_data();

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
        self.is_key(None, KeyState::Down)
    }

    fn is_key_press(&self, key: Key) -> bool {
        self.is_key(Some(key), KeyState::Press)
    }

    fn is_any_key_press(&self) -> bool {
        self.is_key(None, KeyState::Down)
    }

    fn clear(&mut self) {
        self.keys_down.clear();
        self.keys_up.clear();
        self.keys_press.clear();
    }
}
