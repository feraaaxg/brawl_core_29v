use std::sync::Arc;
use bytes::{Buf, BytesMut};
use tokio::sync::Mutex;
use crate::log;
use crate::message::logic_laser_message_factory::LogicLaserMessageFactory;
use crate::networking::client::ClientConnection;

pub struct MessageManager;

impl MessageManager {
    pub async fn receive_message(data: BytesMut, client: Arc<Mutex<ClientConnection>>) {
        if data.len() < 7 {
            log!("недостаточно данных для обработки сообщения");
            return;
        }

        let mut reader = data.clone();
        let msg_id = reader.get_u16();
        let msg_length = ((reader.get_u8() as u32) << 16) | ((reader.get_u8() as u32) << 8) | (reader.get_u8() as u32);
        let msg_version = reader.get_u16();

        let total_message_buffer = if msg_length as usize <= data.len() - 7 {
            BytesMut::from(data.get(7..7 + msg_length as usize).unwrap_or(&[]))
        } else {
            log!(&format!("неверная длина сообщения: {} байт, доступно {}", msg_length, data.len() - 7));
            BytesMut::new()
        };
        LogicLaserMessageFactory::create_message_by_type(msg_id, total_message_buffer, Arc::clone(&client)).await;
    }
}