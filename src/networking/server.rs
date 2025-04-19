use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use crate::logger;
use crate::log;
use std::io;
use crate::networking::session_manager::SessionManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::networking::client::ClientConnection;

pub struct Server {
    listener: TcpListener,
    session_manager: SessionManager,
}

impl Server {
    pub async fn new(address: String, port: String) -> Self {
        let bind_addr = format!("{}:{}", address, port);
        log!(format!("запущен сервер на адресе: {}", bind_addr).as_str());
        Self {
            listener: TcpListener::bind(bind_addr).await.unwrap(),
            session_manager: SessionManager::new(),
        }
    }

    pub async fn start(&mut self) {
        tokio::spawn(async move {
            loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                log!(format!("{}", input.trim()).as_str());
            }
        });
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => self.process(stream, addr).await,
                Err(e) => {
                    log!(format!("ошибка при обработке клиента: {}", e).as_str());
                }
            }
        }
    }

    async fn process(&mut self, socket: TcpStream, addr: SocketAddr) {
        log!(format!("обработка соединения от: {}", addr).as_str());
        let client = ClientConnection::new(socket.into());
        let client_arc = Arc::new(Mutex::new(client));
        self.session_manager.new_session(Arc::clone(&client_arc));
        tokio::spawn(async move {
            if let Err(e) = ClientConnection::handle_client(Arc::clone(&client_arc)).await {
                log!(format!("ошибка обработки клиента: {}", e).as_str());
            }
        });
    }
}