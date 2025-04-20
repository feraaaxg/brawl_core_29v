use std::{fs, io};
use serde::{Deserialize, Serialize};
use crate::networking::server::Server;
use crate::utils::logger;
pub mod utils;
pub mod networking;
pub mod stream;
pub mod message;
mod logic;


#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerConfig{
    ip: String,
    port: String,
}

fn load_config(file_path: &str) -> io::Result<Option<ServerConfig>> {
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => return Err(e),
    };

    let config: ServerConfig = match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(_) => return Ok(None),
    };

    Ok(Some(config))
}

#[tokio::main]
async fn main() {
    let config_file = "config.json";
    let config = load_config(config_file).unwrap();
    let mut server = Server::new("46.173.214.121".to_string(), "9339".to_string()).await;
    server.expect("REASON").start().await;
}
