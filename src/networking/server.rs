use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use crate::logger;
use crate::log;
use std::io;
use crate::networking::session_manager::SessionManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::watch;
use crate::networking::client::ClientConnection;

pub struct Server {
    listener: TcpListener,
    session_manager: SessionManager,
}

impl Server {
    pub async fn new(address: String, port: String) -> io::Result<Self> {
        let bind_addr = format!("{}:{}", address, port);
        let parsed_addr: SocketAddr = bind_addr.parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Неверный формат адреса {}: {}", bind_addr, e),
            )
        })?;
        log!(format!("запущен сервер на адресе: {}", bind_addr).as_str());
        let listener = TcpListener::bind(parsed_addr).await.map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Ошибка привязки к адресу {}: {}", bind_addr, e),
            )
        })?;
        Ok(Self {
            listener,
            session_manager: SessionManager::new(),
        })
    }

    pub async fn start(&mut self) {
        // Создаём канал watch для отслеживания состояния сервера
        let (tx, mut rx) = watch::channel(true);

        // Запускаем задачу для обработки ввода
        tokio::spawn(async move {
            let mut input = String::new();
            loop {
                input.clear();
                match tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::stdin(), &mut input).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        match input.trim() {
                            "exit" => {
                                let _ = tx.send(false); // Отправляем сигнал завершения
                                log!("сервер завершает работу");
                                break;
                            }
                            _ => log!(format!("введена команда: {}", input.trim()).as_str()),
                        }
                    }
                    Err(e) => log!(format!("ошибка чтения ввода: {}", e).as_str()),
                }
            }
        });

        // Основной цикл сервера
        while *rx.borrow() {
            tokio::select! {
                // Проверяем изменение состояния
                _ = rx.changed() => {
                    if !*rx.borrow() {
                        break;
                    }
                }
                // Обрабатываем входящие соединения
                result = self.listener.accept() => {
                    match result {
                        Ok((stream, addr)) => self.process(stream, addr).await,
                        Err(e) => log!(format!("ошибка при обработке клиента: {}", e).as_str()),
                    }
                }
            }
        }
        log!("сервер остановлен");
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