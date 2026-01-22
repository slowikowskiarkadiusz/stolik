#![cfg_attr(not(feature = "std"), no_std)]

use crate::engine::{color_matrix::ColorMatrix, threading_provider::TThread};

struct Engine<T: TThread> {
    update_thread: T,
    fixed_update_thread: T,
    on_frame_finished: fn(color_matrix: &ColorMatrix),
}

impl<T: TThread> Engine<T> {
    pub fn run(&self){
        self.update_thread =
    }
}
