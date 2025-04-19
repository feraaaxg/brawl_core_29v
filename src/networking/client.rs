use std::io;
use tokio::net::TcpStream;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::log;
use crate::message::message_manager::MessageManager;

pub struct ClientConnection {
    pub socket: Mutex<TcpStream>,
    high_id: i32,
    low_id: i32,
}

impl ClientConnection {
    pub fn new(socket: Mutex<TcpStream>) -> Self {
        Self {
            socket,
            high_id: 0,
            low_id: 0,
        }
    }

    pub fn set_high_id(&mut self, high_id: i32) {
        self.high_id = high_id;
    }

    pub fn set_low_id(&mut self, low_id: i32) {
        self.low_id = low_id;
    }

    pub fn get_high_id(&self) -> i32 {
        self.high_id
    }

    pub fn get_low_id(&self) -> i32 {
        self.low_id
    }

    pub async fn handle_client(client: Arc<Mutex<Self>>) -> io::Result<()> {
        let mut buffer = BytesMut::with_capacity(1024);

        loop {
            let mut client_guard = client.lock().await;
            let mut socket = client_guard.socket.lock().await;
            match socket.read_buf(&mut buffer).await {
                Ok(0) => {
                    log!("клиент отключился");
                    return Ok(());
                }
                Ok(_n) => {
                    log!(format!("получены данные от клиента: {} байт", _n).as_str());
                    MessageManager::receive_message(buffer.clone(), Arc::clone(&client)).await;
                    buffer.clear();
                }
                Err(e) => {
                    log!(format!("ошибка чтения данных из сокета: {}", e).as_str());
                    return Err(e);
                }
            }
        }
    }

    pub async fn send_data(&self, data: &[u8]) -> io::Result<()> {
        let mut socket = self.socket.lock().await;
        socket.write_all(data).await?;
        Ok(())
    }
}