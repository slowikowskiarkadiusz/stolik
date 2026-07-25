extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;

use crate::engine::ai::neat_genome::NeatGenome;
use crate::engine::scene::Scene;
use crate::scenes::astro_duel::astro_scene::AstroDuelScene;
use crate::scenes::pong::pong_scene::PongScene;
use crate::scenes::tanks::tanks_scene::TanksScene;
use crate::scenes::tetris::tetris_scene::{TetrisScene, TetrisSceneMode};

pub struct AiConfig {
    pub game_name: String,
    pub input_count: u32,
    pub output_count: u32,
    pub json: &'static str,
    pub scene_factory: fn() -> Box<dyn Scene>,
}

impl AiConfig {
    pub fn get(game_name: &str) -> Self {
        match game_name {
            "pong" => AiConfig {
                game_name: String::from(game_name),
                input_count: 3,
                output_count: 1,
                json: include_str!("../../../neat_genomes/best_pong.json"),
                scene_factory: || Box::new(PongScene::new()),
            },
            "tetris" => AiConfig {
                game_name: String::from(game_name),
                input_count: 208,
                output_count: 5,
                json: include_str!("../../../neat_genomes/best_tetris.json"),
                scene_factory: || Box::new(TetrisScene::new(TetrisSceneMode::AgainstHuman)),
            },
            "tanks" => AiConfig {
                game_name: String::from(game_name),
                input_count: 3,
                output_count: 1,
                json: include_str!("../../../neat_genomes/best_tanks.json"),
                scene_factory: || Box::new(TanksScene::new()),
            },
            "astro-duel" => AiConfig {
                game_name: String::from(game_name),
                input_count: 3,
                output_count: 1,
                json: include_str!("../../../neat_genomes/best_astro-duel.json"),
                scene_factory: || Box::new(AstroDuelScene::new()),
            },
            _ => {
                panic!("Incorrect game name! ({})", game_name);
                AiConfig {
                    game_name: todo!(),
                    input_count: todo!(),
                    output_count: todo!(),
                    json: todo!(),
                    scene_factory: todo!(),
                }
            }
        }
    }

    pub fn load_genome(&self) -> Option<NeatGenome> {
        #[cfg(feature = "std")]
        {
            let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/neat_genomes");
            let json = std::fs::read_to_string(format!("{dir}/best_{}.json", self.game_name)).unwrap_or_default();
            return NeatGenome::from_json(&json);
        }
        #[cfg(not(feature = "std"))]
        NeatGenome::from_json(self.json)
    }

    #[cfg(feature = "std")]
    pub fn save_json(game_name: &str, json: String) -> std::io::Result<()> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/neat_genomes");
        std::fs::write(format!("{dir}/best_{game_name}.json"), json)
    }
}
