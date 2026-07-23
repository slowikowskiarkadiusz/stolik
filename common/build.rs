fn main() {
    std::fs::create_dir_all("neat_genomes").unwrap();
    for name in ["pong", "tetris", "tanks", "astro-duel"] {
        let path = format!("neat_genomes/best_{name}.bin");
        if !std::path::Path::new(&path).exists() {
            std::fs::write(&path, []).unwrap();
        }
        println!("cargo:rerun-if-changed={path}");
    }
}
