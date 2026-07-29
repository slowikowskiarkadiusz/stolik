use common::engine::{
    ai::{ai_config::AiConfig, neat_genome::NeatGenome},
    color_matrix::ColorMatrix,
    engine::{Engine, set_input},
    input::{input::Input, key::KEYS_LENGTH},
    scene::Scene,
};
use rand::{SeedableRng, rngs::SmallRng};
use spin::Mutex;
use std::{env::var, sync::Arc};

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
                json: todo!(),
                scene_factory: todo!(),
                input_factory: todo!(),
            }
        }
    };

    let mut rng = SmallRng::from_entropy();
    let mut population: Vec<NeatGenome> = if let Some(best) = ai_config.load_genome() {
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
        // let population_arc = Arc::new(Mutex::new(population));
        let mut finished_population: Vec<NeatGenome> = Vec::new();

        for pair_index in (0..POPULATION_COUNT).step_by(2) {
            let mut input0 = (ai_config.input_factory)(0);
            let mut input1 = (ai_config.input_factory)(1);
            input0.set_genome(population.pop().unwrap());
            input1.set_genome(population.pop().unwrap());
            set_input(0, input0);
            set_input(1, input1);

            let mut engine = Engine::new(Some(scene_factory()));
            let on_frame: Arc<dyn Fn(&ColorMatrix) + Send + Sync> = Arc::new(|_: &ColorMatrix| {});

            engine.ensure_scene();

            let mut training_timer = TRAINING_DURATION;

            loop {
                let delta_time = 1.0 / 30.0;
                training_timer -= delta_time;
                engine.tick_frame(delta_time, &on_frame);

                if engine.is_game_over() || training_timer <= 0.0 {
                    if let Some(input0) = engine.inputs[0].as_ai_input() {
                        finished_population.push(input0.get_genome().clone());
                    }
                    if let Some(input1) = engine.inputs[1].as_ai_input() {
                        finished_population.push(input1.get_genome().clone());
                    }
                    break;
                }
            }
        }

        population = finished_population;
        // let mut pop = population_arc.lock();
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        println!(
            "generation {}: top fitness: {:.2}  second: {:.2}",
            generation, population[0].fitness, population[1].fitness
        );

        AiConfig::save_json(&ai_config.game_name, population[0].to_json()).ok();

        let evolved = NeatGenome::reproduce(population.drain(..).collect(), POPULATION_COUNT as u8, &mut rng);
        population = evolved;
        generation += 1;
    }
}
