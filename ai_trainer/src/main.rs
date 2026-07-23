use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};

use common::{
    engine::{ai::neat_genome::NeatGenome, color_matrix::ColorMatrix, engine::Engine, input::key::KEYS_LENGTH, scene::Scene},
    scenes::pong::pong_scene::PongScene,
};
use desktop_main::{desktop_input::DesktopInput, desktop_threading_provider::DesktopThread};
use minifb::Key;

use crate::ai_input::AiInput;

const POPULATION_COUNT: usize = 10;

pub mod ai_input;

struct AiTrainingConfig {
    input_count: u32,
    output_count: u32,
    scene_fn: Box<dyn FnOnce() -> Box<dyn Scene> + Send + Sync>,
}

fn main() {
    let setup = get_pong_config();
    let population: Vec<NeatGenome> = (0..POPULATION_COUNT)
        .map(|_| NeatGenome::new(setup.input_count, setup.output_count))
        .collect();

    let population = Arc::new(Mutex::new(population));

    let mut handles: Vec<JoinHandle<f64>> = Vec::new();

    for pair_index in (0..POPULATION_COUNT).step_by(2) {
        let population = Arc::clone(&population);

        let handle = thread::spawn(move || {
            let p0_input: Arc<Mutex<[bool; KEYS_LENGTH as usize]>> = Arc::new(Mutex::new([false; KEYS_LENGTH as usize]));
            let p1_input: Arc<Mutex<[bool; KEYS_LENGTH as usize]>> = Arc::new(Mutex::new([false; KEYS_LENGTH as usize]));

            let mut engine = Engine::new(
                [Box::new(AiInput::new(p0_input.clone())), Box::new(AiInput::new(p1_input.clone()))],
                Some(Box::new(PongScene::new(false))),
            );

            let on_frame_func: Arc<dyn Fn(&ColorMatrix) + Send + Sync> = Arc::new(|_mat: &ColorMatrix| {});
            engine.ensure_scene();

            main_loop = loop {
                let ai_data = engine.get_scene_data_for_ai();

                let (p0_outputs, p1_outputs) = {
                    let mut pop = population.lock().unwrap();
                    let p0_outputs = pop[pair_index].activate(ai_data.inputs[0].clone());
                    pop[pair_index].fitness = ai_data.points[0];
                    let p1_outputs = pop[pair_index + 1].activate(ai_data.inputs[1].clone());
                    pop[pair_index + 1].fitness = ai_data.points[1];
                    (p0_outputs, p1_outputs)
                };

                *p0_input.lock().unwrap() = [
                    false,
                    false,
                    p0_outputs[0] > 0.5,
                    p0_outputs[0] < 0.5,
                    false,
                    false,
                    false,
                    false,
                    false,
                ];
                *p1_input.lock().unwrap() = [
                    false,
                    false,
                    p1_outputs[0] > 0.5,
                    p1_outputs[0] < 0.5,
                    false,
                    false,
                    false,
                    false,
                    false,
                ];

                engine.tick_frame(1.0 / 33.0, &on_frame_func);

                if engine.is_game_over() {
                    break main_loop;
                }
            };

            let (p0_fitness, p1_fitness) = {
                let mut pop = population.lock().unwrap();
                let p0_fitness = pop[pair_index].get_fitness();
                let p1_fitness = pop[pair_index + 1].get_fitness();
                (p0_fitness, p1_fitness)
            };

            [p0_fitness, p1_fitness]
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn get_pong_config() -> AiTrainingConfig {
    AiTrainingConfig {
        input_count: 2,
        output_count: 1,
        scene_fn: Box::new(|| Box::new(PongScene::new(false))),
    }
}
