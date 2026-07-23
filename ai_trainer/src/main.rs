use crate::ai_input::AiInput;
use common::{
    engine::{ai::neat_genome::NeatGenome, color_matrix::ColorMatrix, engine::Engine, input::key::KEYS_LENGTH},
    scenes::pong::pong_scene::PongScene,
};
use rand::{SeedableRng, rngs::SmallRng};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub mod ai_input;

const POPULATION_COUNT: usize = 10;
const INPUT_COUNT: u32 = 3;
const OUTPUT_COUNT: u32 = 1;

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut population: Vec<NeatGenome> = (0..POPULATION_COUNT).map(|_| NeatGenome::new(INPUT_COUNT, OUTPUT_COUNT)).collect();

    let mut generation = 0u32;
    loop {
        let population_arc = Arc::new(Mutex::new(population));
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for pair_index in (0..POPULATION_COUNT).step_by(2) {
            let population_arc = Arc::clone(&population_arc);
            let handle = thread::spawn(move || {
                let p0_held: Arc<Mutex<[bool; KEYS_LENGTH as usize]>> = Arc::new(Mutex::new([false; KEYS_LENGTH as usize]));
                let p1_held: Arc<Mutex<[bool; KEYS_LENGTH as usize]>> = Arc::new(Mutex::new([false; KEYS_LENGTH as usize]));

                let mut engine = Engine::new(
                    [Box::new(AiInput::new(p0_held.clone())), Box::new(AiInput::new(p1_held.clone()))],
                    Some(Box::new(PongScene::new(false))),
                );
                let on_frame: Arc<dyn Fn(&ColorMatrix) + Send + Sync> = Arc::new(|_: &ColorMatrix| {});

                engine.ensure_scene();

                loop {
                    let ai_data = engine.get_scene_data_for_ai();

                    if ai_data.inputs[0].is_empty() || ai_data.inputs[1].is_empty() {
                        engine.tick_frame(1.0 / 33.0, &on_frame);
                        if engine.is_game_over() {
                            break;
                        }
                        continue;
                    }

                    let (p0_outputs, p1_outputs) = {
                        let mut pop = population_arc.lock().unwrap_or_else(|e| e.into_inner());
                        let p0_out = pop[pair_index].activate(ai_data.inputs[0].clone());
                        pop[pair_index].fitness = ai_data.points[0];
                        let p1_out = pop[pair_index + 1].activate(ai_data.inputs[1].clone());
                        pop[pair_index + 1].fitness = ai_data.points[1];
                        (p0_out, p1_out)
                    };

                    *p0_held.lock().unwrap() = keys_from_outputs(&p0_outputs);
                    *p1_held.lock().unwrap() = keys_from_outputs(&p1_outputs);

                    engine.tick_frame(1.0 / 33.0, &on_frame);

                    if engine.is_game_over() {
                        break;
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut pop = population_arc.lock().unwrap();
        pop.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        println!(
            "generation {}: top fitness: {:.2}  second: {:.2}",
            generation, pop[0].fitness, pop[1].fitness
        );

        let evolved = NeatGenome::reproduce(pop.drain(..).collect(), POPULATION_COUNT as u8, &mut rng);
        drop(pop);
        population = evolved;
        generation += 1;
    }
}

fn keys_from_outputs(outputs: &[f64]) -> [bool; KEYS_LENGTH as usize] {
    let mut keys = [false; KEYS_LENGTH as usize];
    if let Some(&v) = outputs.first() {
        keys[2] = v > 0.5;
        keys[3] = v < 0.5;
    }
    keys
}
