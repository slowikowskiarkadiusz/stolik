use common::engine::{
    ai::{ai_config::AiConfig, ai_input::AiInput, neat_genome::NeatGenome},
    color_matrix::ColorMatrix,
    engine::Engine,
    input::key::KEYS_LENGTH,
    scene::Scene,
};
use rand::{SeedableRng, rngs::SmallRng};
use spin::Mutex;
use std::{
    env::{self, var},
    fs,
    sync::Arc,
    thread::{self, JoinHandle},
};

const POPULATION_COUNT: usize = 10;
const TRAINING_DURATION: f32 = 30.0;

fn main() {
    let game = var("game");
    let mut ai_config: AiConfig = match game {
        Ok(game_name) => AiConfig::get(&game_name),
        Err(err) => {
            panic!("{}", err);
            AiConfig {
                game_name: todo!(),
                input_count: todo!(),
                output_count: todo!(),
                bytes: todo!(),
                scene_factory: todo!(),
            }
        }
    };

    let mut rng = SmallRng::from_entropy();
    let mut population: Vec<NeatGenome> = if let Some(best) = NeatGenome::from_bytes(ai_config.bytes) {
        let mut pop = vec![best.clone()];
        while pop.len() < POPULATION_COUNT {
            let mut genome = best.clone();
            genome.mutate(&mut rng);
            pop.push(genome);
        }
        pop
    } else {
        (0..POPULATION_COUNT)
            .map(|_| NeatGenome::new(ai_config.input_count, ai_config.output_count))
            .collect()
    };

    let scene_factory = ai_config.scene_factory;
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
                    Some(scene_factory()),
                );
                let on_frame: Arc<dyn Fn(&ColorMatrix) + Send + Sync> = Arc::new(|_: &ColorMatrix| {});

                engine.ensure_scene();

                let mut training_timer = TRAINING_DURATION;

                loop {
                    let ai_data = engine.get_scene_data_for_ai();

                    if !ai_data.inputs[0].is_empty() && !ai_data.inputs[1].is_empty() {
                        let (p0_outputs, p1_outputs) = {
                            let mut pop = population_arc.lock();
                            let p0_out = pop[pair_index].activate(ai_data.inputs[0].clone());
                            pop[pair_index].fitness = ai_data.points[0];
                            let p1_out = pop[pair_index + 1].activate(ai_data.inputs[1].clone());
                            pop[pair_index + 1].fitness = ai_data.points[1];
                            // println!(
                            //     "[{}/{}] {:.0}  [{}/{}] {:.0}",
                            //     pair_index,
                            //     POPULATION_COUNT,
                            //     ai_data.points[0],
                            //     pair_index + 1,
                            //     POPULATION_COUNT,
                            //     ai_data.points[1]
                            // );
                            (p0_out, p1_out)
                        };

                        *p0_held.lock() = (ai_data.outputs_to_keys)(&p0_outputs);
                        *p1_held.lock() = (ai_data.outputs_to_keys)(&p1_outputs);
                    }

                    let delta_time = 1.0 / 30.0;
                    training_timer -= delta_time;
                    engine.tick_frame(delta_time, &on_frame);

                    if engine.is_game_over() || training_timer <= 0.0 {
                        break;
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut pop = population_arc.lock();
        pop.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        println!(
            "generation {}: top fitness: {:.2}  second: {:.2}",
            generation, pop[0].fitness, pop[1].fitness
        );

        AiConfig::save_bytes(&ai_config.game_name, pop[0].to_bytes()).ok();

        let evolved = NeatGenome::reproduce(pop.drain(..).collect(), POPULATION_COUNT as u8, &mut rng);
        drop(pop);
        population = evolved;
        generation += 1;
    }
}
