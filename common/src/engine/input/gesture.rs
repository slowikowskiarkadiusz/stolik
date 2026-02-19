extern crate alloc;
use crate::engine::{
    hash_map::HashMap,
    input::key::{KEYS_LENGTH, Key, u8_to_key},
    max_heap::MaxHeap,
};
use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::array;
use embassy_time::Instant;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum State {
    Down,
    Up,
    Press,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Gesture {
    Once = 1,
    Twice = 2,
    Trice = 3,
    Prolonged = 4,
    Repeating = 5,
}

fn u8_to_gesture(g: u8) -> Gesture {
    match g {
        1 => Gesture::Once,
        2 => Gesture::Twice,
        3 => Gesture::Trice,
        4 => Gesture::Prolonged,
        5 => Gesture::Repeating,
        _ => panic!("cant map {} to gesture", g),
    }
}

const STATES: [State; 3] = [State::Down, State::Up, State::Press];
const MAX_GESTURE_DELAY: f32 = 200.0;
const LONG_PRESS_DURATION: f32 = 300.0;
const REPEATER_PRESS_DURATION: f32 = 150.0;

pub struct Gestures {
    states: HashMap<State, Box<dyn Fn(Key) -> bool>>,
    last_action_timestamps: [HashMap<State, MaxHeap>; KEYS_LENGTH as usize],
    gestures_this_frame: HashMap<String, bool>,
    single_gestures_this_frame: HashMap<String, Gesture>,
    press_timers: HashMap<u8, f32>,
    repeater_timers: HashMap<u8, f32>,
}

impl Gestures {
    pub fn new() -> Self {
        Self {
            states: HashMap::<State, Box<dyn Fn(Key) -> bool>>::new(),
            last_action_timestamps: array::from_fn(|_| {
                let mut hash_map = HashMap::<State, MaxHeap>::new();
                for state in STATES {
                    hash_map[&state] = MaxHeap::new(3);
                }
                hash_map
            }),
            gestures_this_frame: HashMap::new(),
            single_gestures_this_frame: HashMap::new(),
            press_timers: HashMap::new(),
            repeater_timers: HashMap::new(),
        }
    }

    pub fn make_key(key: Key, state: State, gesture: Option<Gesture>) -> String {
        let mut res = format!("{}_{}", key as u8, state as u8);
        if let Some(ges) = gesture {
            res = format!("{}_{}", res, ges as u8);
        }
        res
    }

    pub fn is(&self, key: Key, state: State, gesture: Gesture, single: Option<bool>) -> bool {
        let single_value = single.unwrap_or(false);
        let id = Gestures::make_key(key, state, if single_value { None } else { Some(gesture.clone()) });
        if single_value {
            self.single_gestures_this_frame[&id] == gesture
        } else {
            self.gestures_this_frame[&id]
        }
    }

    pub fn are(&self, keys: Vec<Key>, state: State, gesture: Gesture, single: Option<bool>) -> bool {
        for k in keys {
            if !self.is(k, state.clone(), gesture.clone(), single) {
                return false;
            }
        }

        true
    }

    pub fn tick(&mut self, delta_time: f32) {
        let now = Instant::now().as_millis();

        for k in 0..KEYS_LENGTH {
            let key: Key = u8_to_key(k);

            for (state, pred) in &self.states {
                if pred(key) {
                    // tu moze jednak powinienem uzywac u64
                    self.last_action_timestamps[k as usize][state].insert(now as u32);

                    for g in Gesture::Trice as u8..Gesture::Once as u8 {
                        let values = self.last_action_timestamps[k as usize][state].values();

                        if (values.len() as u8) < g {
                            continue;
                        }

                        let mut is_valid = true;
                        for i in 0..g {
                            if (values[i as usize] - values[i as usize + 1]) as f32 > MAX_GESTURE_DELAY {
                                is_valid = false;
                                break;
                            }
                        }

                        if !is_valid {
                            continue;
                        }

                        let g_key = Gestures::make_key(key, state.clone(), Some(u8_to_gesture(g)));
                        self.gestures_this_frame[&g_key] = true;
                    }
                }

                if state == &State::Press {
                    let value = if pred(key) { delta_time } else { -self.press_timers[&k] };
                    self.press_timers[&k] += value;
                    self.repeater_timers[&k] += value;

                    let press_key = Gestures::make_key(key, state.clone(), None);
                    if self.press_timers[&k] > LONG_PRESS_DURATION {
                        self.gestures_this_frame[&Gestures::make_key(key, state.clone(), Some(Gesture::Prolonged))] = true;
                        self.single_gestures_this_frame[&press_key] = Gesture::Prolonged;
                        self.press_timers[&k] = 0.0;
                    }

                    if self.press_timers[&k] > REPEATER_PRESS_DURATION {
                        self.gestures_this_frame[&Gestures::make_key(key, state.clone(), Some(Gesture::Repeating))] = true;
                        self.single_gestures_this_frame[&press_key] = Gesture::Repeating;
                        self.repeater_timers[&k] = 0.0;
                    }
                }
            }
        }
    }

    pub fn late_tick(&mut self) {
        self.gestures_this_frame.clear();
        self.single_gestures_this_frame.clear();
    }
}
