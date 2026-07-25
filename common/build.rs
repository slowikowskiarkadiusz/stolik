fn main() {
    std::fs::create_dir_all("neat_genomes").unwrap();
    for name in ["pong", "tetris", "tanks", "astro-duel"] {
        let json_path = format!("neat_genomes/best_{name}.json");
        if !std::path::Path::new(&json_path).exists() {
            std::fs::write(&json_path, "").unwrap();
        }
        println!("cargo:rerun-if-changed={json_path}");
    }
}
