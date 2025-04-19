use std::sync::Arc;
use bytes::BytesMut;
use tokio::sync::Mutex;
use crate::log;
use crate::message::client::client_hello_message::ClientHelloMessage;
use crate::message::client::titan_login_message::TitanLoginMessage;
use crate::message::piranha_message::FactoryMessage;
use crate::message::server::authentication_response_message::AuthenticationResponseMessage;
use crate::networking::client::ClientConnection;

pub struct LogicLaserMessageFactory;

impl LogicLaserMessageFactory {
    pub async fn create_message_by_type(
        message_type: u16,
        buffer: BytesMut,
        client: Arc<Mutex<ClientConnection>>,
    ){

        match message_type {
            10100 => Self::process_message::<ClientHelloMessage>(buffer, client).await,
            10101 => Self::process_message::<TitanLoginMessage>(buffer, client).await,
            20104 => Self::process_message::<AuthenticationResponseMessage>(buffer, client).await,
            _ => {
                log!(&format!("неизвестный тип сообщения: {}", message_type));
            }
        }
    }

    async fn process_message<M: FactoryMessage>(
        buffer: BytesMut,
        client: Arc<Mutex<ClientConnection>>,
    ) {
        let mut message = M::new(buffer, client);
        log!(&format!("обрабатывается сообщение: {}", message.get_message_type_name()));
        message.decode().await;
        message.process().await;
        message.encode().await;
    }
}