use std::path::PathBuf;

use tower_http::services::ServeDir;

pub fn get_broadcast_port() -> u16 {
    return dotenv::var("BACKEND_PORT").unwrap_or("3000".to_string()).parse().unwrap();
}

pub fn create_assets_service() -> ServeDir {
    let public_dir = PathBuf::from("public");
    let build_dir = PathBuf::from("../build");

    if public_dir.exists() {
        return ServeDir::new(public_dir);
    } else if build_dir.exists() {
        return ServeDir::new(build_dir);
    } else {
        panic!("No asset directory found");
    }
}

pub fn show_brand_message() {
    println!(
        "
 ██████  ██████  ███    ██ ██    ██  ██████  ██    ██
██      ██    ██ ████   ██ ██    ██ ██    ██  ██  ██
██      ██    ██ ██ ██  ██ ██    ██ ██    ██   ████
██      ██    ██ ██  ██ ██  ██  ██  ██    ██    ██
 ██████  ██████  ██   ████   ████    ██████     ██
    "
    );
    println!(
        "Convoy Terminal\nVersion: {}\n",
        env!("CARGO_PKG_VERSION")
    );
    println!("View the source code at https://github.com/convoypanel/coterm\n\n\n");
}