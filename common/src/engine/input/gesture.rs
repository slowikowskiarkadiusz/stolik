extern crate alloc;
use crate::engine::{
    hash_map::HashMap,
    input::{
        input::{Input, InputSnapshot},
        key::{KEYS_LENGTH, Key, u8_to_key},
    },
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
const MAX_GESTURE_DELAY_SEC: f32 = 0.2;
const LONG_PRESS_DURATION_SEC: f32 = 0.3;
const REPEATER_PRESS_DURATION_SEC: f32 = 0.15;

pub struct Gestures {
    states: HashMap<State, Box<dyn Fn(Key, &InputSnapshot) -> bool>>,
    last_action_timestamps: [HashMap<State, MaxHeap>; KEYS_LENGTH as usize],
    gestures_this_frame: HashMap<String, bool>,
    single_gestures_this_frame: HashMap<String, Gesture>,
    press_timers_sec: HashMap<u8, f32>,
    repeater_timers_sec: HashMap<u8, f32>,
}

impl Gestures {
    pub fn new() -> Self {
        let mut states = HashMap::<State, Box<dyn Fn(Key, &InputSnapshot) -> bool>>::new();
        states.insert(State::Down, Box::new(|key, snapshot| snapshot[&key].is_down));
        states.insert(State::Press, Box::new(|key, snapshot| snapshot[&key].is_press));
        states.insert(State::Up, Box::new(|key, snapshot| snapshot[&key].is_up));

        Self {
            states,
            last_action_timestamps: array::from_fn(|_| {
                let mut hash_map = HashMap::<State, MaxHeap>::new();
                for state in STATES {
                    hash_map.insert(state, MaxHeap::new(3));
                }
                hash_map
            }),
            gestures_this_frame: HashMap::new(),
            single_gestures_this_frame: HashMap::new(),
            press_timers_sec: HashMap::new(),
            repeater_timers_sec: HashMap::new(),
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
            if self.single_gestures_this_frame.contains_key(&id) {
                self.single_gestures_this_frame[&id] == gesture
            } else {
                false
            }
        } else {
            if self.gestures_this_frame.contains_key(&id) {
                self.gestures_this_frame[&id]
            } else {
                false
            }
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

    pub fn tick(&mut self, input: InputSnapshot, delta_time: f32) {
        let now = Instant::now().as_millis();

        for k in 0..KEYS_LENGTH {
            let key: Key = u8_to_key(k);

            // if key == Key::P1Left {
            //     println!("gkey: {}", serde_json::to_string(&input[&key]).unwrap());
            // }

            for (state, pred) in &self.states {
                let pred_value = pred(key, &input);

                if pred_value {
                    // TODO tu moze jednak powinienem uzywac u64
                    self.last_action_timestamps[k as usize][state].insert(now as u32);

                    for g in (Gesture::Once as u8..Gesture::Trice as u8).rev() {
                        // if key == Key::P1Left {
                        //     println!("state: {}, pred: {}", state.clone() as u8, pred_value);
                        // }
                        let values = self.last_action_timestamps[k as usize][state].values();

                        // println!("{}", values.len());

                        if (values.len() as u8) < g {
                            continue;
                        }

                        let mut is_valid = true;
                        for i in 0..g - 1 {
                            if (values[i as usize] as f32 - values[i as usize + 1] as f32) as f32 > MAX_GESTURE_DELAY_SEC {
                                is_valid = false;
                                break;
                            }
                        }

                        if !is_valid {
                            continue;
                        }

                        let g_key = Gestures::make_key(key, state.clone(), Some(u8_to_gesture(g)));
                        self.gestures_this_frame.insert(g_key, true);
                    }
                }

                if state == &State::Press {
                    if !self.press_timers_sec.contains_key(&k) {
                        self.press_timers_sec.insert(k, 0.0);
                    }
                    if !self.repeater_timers_sec.contains_key(&k) {
                        self.repeater_timers_sec.insert(k, 0.0);
                    }
                    let value = if pred_value { delta_time } else { -self.press_timers_sec[&k] };
                    self.press_timers_sec[&k] += value;
                    self.repeater_timers_sec[&k] += value;

                    let press_key = Gestures::make_key(key, state.clone(), None);
                    if self.press_timers_sec[&k] > LONG_PRESS_DURATION_SEC {
                        let gesture_key = Gestures::make_key(key, state.clone(), Some(Gesture::Prolonged));
                        if !self.gestures_this_frame.contains_key(&gesture_key) {
                            self.gestures_this_frame.insert(gesture_key.clone(), true);
                        }
                        if !self.single_gestures_this_frame.contains_key(&press_key) {
                            self.single_gestures_this_frame.insert(press_key.clone(), Gesture::Prolonged);
                        }

                        self.gestures_this_frame[&gesture_key] = true;
                        self.single_gestures_this_frame[&press_key] = Gesture::Prolonged;
                        self.press_timers_sec[&k] = 0.0;
                    }

                    if self.press_timers_sec[&k] > REPEATER_PRESS_DURATION_SEC {
                        let gesture_key = Gestures::make_key(key, state.clone(), Some(Gesture::Repeating));
                        if !self.gestures_this_frame.contains_key(&gesture_key) {
                            self.gestures_this_frame.insert(gesture_key.clone(), true);
                        }
                        if !self.single_gestures_this_frame.contains_key(&press_key) {
                            self.single_gestures_this_frame.insert(press_key.clone(), Gesture::Repeating);
                        }

                        self.gestures_this_frame[&gesture_key] = true;
                        self.single_gestures_this_frame[&press_key] = Gesture::Repeating;
                        self.repeater_timers_sec[&k] = 0.0;
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
