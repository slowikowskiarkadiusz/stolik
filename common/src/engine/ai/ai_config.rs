extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::engine::scene::Scene;
use crate::scenes::astro_duel::astro_scene::AstroDuelScene;
use crate::scenes::pong::pong_scene::PongScene;
use crate::scenes::tanks::tanks_scene::TanksScene;
use crate::scenes::tetris::tetris_scene::{TetrisScene, TetrisSceneMode};

pub struct AiConfig {
    pub game_name: String,
    pub input_count: u32,
    pub output_count: u32,
    pub bytes: &'static [u8],
    pub scene_factory: fn() -> Box<dyn Scene>,
}

impl AiConfig {
    pub fn get(game_name: &str) -> Self {
        match game_name {
            "pong" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_pong.json"),
                scene_factory: || Box::new(PongScene::new()),
            },
            "tetris" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 208,
                output_count: 5,
                bytes: include_bytes!("../../../neat_genomes/best_tetris.json"),
                scene_factory: || Box::new(TetrisScene::new(TetrisSceneMode::AgainstHuman)),
            },
            "tanks" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_tanks.json"),
                scene_factory: || Box::new(TanksScene::new()),
            },
            "astro-duel" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_astro-duel.json"),
                scene_factory: || Box::new(AstroDuelScene::new()),
            },
            _ => {
                panic!("Incorrect game name! ({})", game_name);
                AiConfig {
                    game_name: todo!(),
                    input_count: todo!(),
                    output_count: todo!(),
                    bytes: todo!(),
                    scene_factory: todo!(),
                }
            }
        }
    }

    #[cfg(feature = "std")]
    pub fn save_bytes(game_name: &str, bytes: Vec<u8>) -> std::io::Result<()> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/neat_genomes");
        std::fs::write(format!("{dir}/best_{game_name}.json"), bytes)
    }
}
