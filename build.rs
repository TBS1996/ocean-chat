use std::{fs::File, path::Path};

fn main() {
    create_file("config.toml".to_string());
    create_file("./files/dist".to_string());
    create_file("./files/scores".to_string());
}

fn create_file(file_path: String) {
    let path = Path::new(&file_path);

    if !path.exists() {
        File::create(path).unwrap();
    }
}
