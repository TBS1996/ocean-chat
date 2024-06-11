use std::{fs::File, path::Path};

fn main() {
    let path = Path::new("config.toml");

    if !path.exists() {
        File::create(path).unwrap();
    }
}
