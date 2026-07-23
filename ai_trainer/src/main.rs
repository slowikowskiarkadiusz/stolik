use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use common::{
    engine::{ai::neat_genome::NeatGenome, color_matrix::ColorMatrix, engine::Engine, scene::Scene},
    scenes::pong::pong_scene::PongScene,
};
use desktop_main::{desktop_input::DesktopInput, desktop_threading_provider::DesktopThread};
use minifb::Key;

const POPULATION_COUNT: usize = 10;

struct AiTrainingConfig {
    input_count: u32,
    output_count: u32,
    scene_fn: Box<dyn FnOnce() -> Box<dyn Scene> + Send + Sync>,
}

fn main() {
    let setup = get_pong_config();

    let mut population: Vec<NeatGenome> = Vec::new();
    for population_index in 0..POPULATION_COUNT {
        population.push(NeatGenome::new(setup.input_count, setup.output_count));
    }

    let mut population_index = 0;
    while population_index < POPULATION_COUNT {
        let input_state = Arc::new(Mutex::new(init_input_state()));
        let cloned_input_state = input_state.clone();
        let mut engine = Engine::new(
            Box::new(DesktopInput::new(cloned_input_state)),
            Some(Box::new(PongScene::new(false))),
        );
        let on_frame_func: Arc<dyn Fn(&Matrix<Color>) + Send + Sync> = Arc::new(|mat: &ColorMatrix| {});

        let loop_cloned_input_state = input_state.clone();

        engine.ensure_scene();

        loop {
            let ai_data = engine.get_scene_ai_inputs();

            let p0_outputs = population[population_index].activate(ai_data[0]);
            population_index += 1;
            let p1_outputs = population[population_index].activate(ai_data[1]);
            population_index += 1;

            let a = loop_cloned_input_state.lock().as_mut().unwrap();
            a[Key::A] = (true, true);

            engine.tick_frame(1.00 / 33.0, &on_frame_func);
            engine.input.as_mut().late_update(dt);
        }
    }
}

fn get_pong_config() -> AiTrainingConfig {
    AiTrainingConfig {
        input_count: 2,
        output_count: 1,
        scene_fn: Box::new(|| Box::new(PongScene::new(false))),
    }
}

fn init_input_state() -> HashMap<Key, (bool, bool)> {
    let mut input_state = HashMap::new();
    input_state.insert(Key::Space, (false, false));
    input_state.insert(Key::S, (false, false));
    input_state.insert(Key::W, (false, false));
    input_state.insert(Key::A, (false, false));
    input_state.insert(Key::D, (false, false));
    input_state.insert(Key::F, (false, false));
    input_state.insert(Key::G, (false, false));
    input_state.insert(Key::Down, (false, false));
    input_state.insert(Key::Up, (false, false));
    input_state.insert(Key::Left, (false, false));
    input_state.insert(Key::Right, (false, false));
    input_state.insert(Key::O, (false, false));
    input_state.insert(Key::P, (false, false));
    input_state
}
