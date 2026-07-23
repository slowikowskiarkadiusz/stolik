extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub struct AiConfig {
    pub game_name: String,
    pub input_count: u32,
    pub output_count: u32,
    pub bytes: &'static [u8],
}

impl AiConfig {
    pub fn get(game_name: &str) -> Self {
        match game_name {
            "pong" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_pong.bin"),
            },
            "tetris" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_tetris.bin"),
            },
            "tanks" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_tanks.bin"),
            },
            "astro-duel" => AiConfig {
                game_name: game_name.to_owned(),
                input_count: 3,
                output_count: 1,
                bytes: include_bytes!("../../../neat_genomes/best_astro-duel.bin"),
            },
            _ => {
                panic!("Incorrect game name! ({})", game_name)
            }
        }
    }

    #[cfg(feature = "std")]
    pub fn save_bytes(game_name: &str, bytes: Vec<u8>) -> std::io::Result<()> {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/neat_genomes");
        std::fs::write(format!("{dir}/best_{game_name}.bin"), bytes)
    }
}
