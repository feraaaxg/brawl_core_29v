use crate::networking::server::Server;
use crate::utils::logger;
pub mod utils;
pub mod networking;
pub mod stream;
pub mod message;

#[tokio::main]
async fn main() {
    let host = "192.168.1.136".to_string();
    let port = "9339".to_string();
    let mut server = Server::new(host, port).await;
    server.start().await;
}
