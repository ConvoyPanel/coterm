use std::path::PathBuf;
use dotenv::var;

use tower_http::services::ServeDir;
use tracing::warn;

pub fn get_broadcast_port() -> u16 {
    return var("BACKEND_PORT").unwrap_or("2115".to_string()).parse().unwrap();
}

pub fn create_assets_service() -> ServeDir {
    let public_dir = PathBuf::from("public");
    let build_dir = PathBuf::from("../build");
    let static_dir = PathBuf::from("../static");

    if public_dir.exists() {
        return ServeDir::new(public_dir);
    } else if build_dir.exists() {
        return ServeDir::new(build_dir);
    } else if static_dir.exists() {
        warn!("Using static directory for assets. This is not recommended for production use.");

        return ServeDir::new(static_dir);
    } else {
        panic!("No viable asset directory found");
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