#![no_std]
#![no_main]
use core::prelude::v1::*;

use embassy_time::Duration;
use esp_hal as _;
use common::engine::threading_provider::Thread;
extern crate alloc;
use alloc::boxed::Box;
use core::ops::Fn;
use core::marker::Send;

pub struct Esp32Thread;

impl Thread for Esp32Thread {
    fn start(handle: Box<dyn Fn() + Send>) -> Esp32Thread {
        // Embassy nie pozwala spawnować dynamicznie z Box<dyn Fn>
        // musisz spawnować taski statycznie przez #[embassy_executor::task]
        // Tu możesz tylko wywołać handle bezpośrednio lub użyć drugiego core
        handle();
        Esp32Thread
    }

    fn sleep_for(ms: u64) {
    }

    fn stop(&self) {}
}
