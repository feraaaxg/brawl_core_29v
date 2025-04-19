use std::sync::Arc;
use bytes::{BufMut, BytesMut};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use crate::log;
use crate::networking::client::ClientConnection;
use crate::stream::byte_stream::ByteStream;


pub trait MessageOps {
    fn get_stream(&mut self) -> &mut ByteStream;
    fn get_client(&self) -> Arc<Mutex<ClientConnection>>;
    fn get_message_id(&self) -> u16;
    fn set_message_id(&mut self, id: u16);
    fn get_message_version(&self) -> u16;
    fn set_message_version(&mut self, version: u16);
    fn get_message_type_name(&self) -> &str;

    fn read_int(&mut self) -> i32 {
        self.get_stream().read_int()
    }

    fn write_int(&mut self, value: i32) {
        self.get_stream().write_int(value)
    }

    fn read_short(&mut self) -> i16 {
        self.get_stream().read_short()
    }

    fn write_short(&mut self, value: i16) {
        self.get_stream().write_short(value)
    }

    fn read_string(&mut self) -> String {
        self.get_stream().read_string()
    }

    fn write_string(&mut self, value: Option<&str>) {
        self.get_stream().write_string(value)
    }

    fn read_vint(&mut self) -> i32 {
        self.get_stream().read_vint()
    }

    fn write_vint(&mut self, value: i32) {
        self.get_stream().write_vint(value)
    }

    fn read_boolean(&mut self) -> bool {
        self.get_stream().read_boolean()
    }

    fn write_boolean(&mut self, value: bool) {
        self.get_stream().write_boolean(value)
    }

    fn read_data_reference(&mut self) -> [i32; 2] {
        self.get_stream().read_data_reference()
    }

    fn write_data_reference(&mut self, value1: i32, value2: i32) {
        self.get_stream().write_data_reference(value1, value2)
    }

    fn read_logic_long(&mut self) -> [i32; 2] {
        self.get_stream().read_logic_long()
    }

    fn write_logic_long(&mut self, value1: i32, value2: i32) {
        self.get_stream().write_logic_long(value1, value2)
    }

    fn read_long(&mut self) -> [i32; 2] {
        self.get_stream().read_long()
    }

    fn write_long(&mut self, value1: i32, value2: i32) {
        self.get_stream().write_long(value1, value2)
    }

    fn write_long_long(&mut self, value: i64) {
        self.get_stream().write_long_long(value)
    }

    fn write_byte(&mut self, value: u8) {
        self.get_stream().write_byte(value)
    }

    fn write_bytes(&mut self, buffer: &[u8]) {
        self.get_stream().write_bytes(buffer)
    }

    fn reset(&mut self) {
        self.get_stream().reset()
    }

    fn get_length(&mut self) -> usize {
        self.get_stream().get_length()
    }

    fn get_buffer(&mut self) -> &BytesMut {
        self.get_stream().get_buffer()
    }

    fn replace_buffer(&mut self, buffer: BytesMut) -> BytesMut {
        self.get_stream().replace_buffer(buffer)
    }

    async fn send(&mut self) -> std::io::Result<()> {
        let mut header = BytesMut::with_capacity(7);
        header.put_u16(self.get_message_id());
        let length = self.get_length() as u32;
        header.put_u8(((length >> 16) & 0xFF) as u8);
        header.put_u8(((length >> 8) & 0xFF) as u8);
        header.put_u8((length & 0xFF) as u8);
        header.put_u16(self.get_message_version());

        let mut data = BytesMut::new();
        data.extend_from_slice(&header);
        data.extend_from_slice(self.get_buffer());
        data.extend_from_slice(&[0xFF, 0xFF, 0x0, 0x0, 0x0, 0x0, 0x0]);

        let client = self.get_client();
        let mut client_guard = client.lock().await;
        let mut socket = client_guard.socket.lock().await;
        socket.write_all(&data).await
    }

    fn is_server_message(id: u16) -> bool {
        id >= 20_000
    }
}

// Core message struct
pub struct PiranhaMessage {
    stream: ByteStream,
    message_id: u16,
    message_version: u16,
    client: Arc<Mutex<ClientConnection>>,
    message_type_name: String,
}

impl PiranhaMessage {
    pub fn new(bytes: BytesMut, client: Arc<Mutex<ClientConnection>>, msg_type_name: &str) -> Self {
        let mut stream = ByteStream::new();
        stream.replace_buffer(bytes);

        log!(&format!("Создано сообщение: {}", msg_type_name));

        PiranhaMessage {
            stream,
            message_id: 20_000,
            message_version: 0,
            client,
            message_type_name: msg_type_name.to_string(),
        }
    }
}

impl MessageOps for PiranhaMessage {
    fn get_stream(&mut self) -> &mut ByteStream {
        &mut self.stream
    }

    fn get_client(&self) -> Arc<Mutex<ClientConnection>> {
        Arc::clone(&self.client)
    }

    fn get_message_id(&self) -> u16 {
        self.message_id
    }

    fn set_message_id(&mut self, id: u16) {
        self.message_id = id;
    }

    fn get_message_version(&self) -> u16 {
        self.message_version
    }

    fn set_message_version(&mut self, version: u16) {
        self.message_version = version;
    }

    fn get_message_type_name(&self) -> &str {
        &self.message_type_name
    }
}

pub trait Coder {
    async fn encode(&mut self) -> &mut Self;
    async fn decode(&mut self) -> &mut Self;
    async fn process(&mut self) -> &mut Self;
}

pub trait FactoryMessage: Coder + MessageOps {
    fn new(buffer: BytesMut, client: Arc<Mutex<ClientConnection>>) -> Self;
}

#[macro_export]
macro_rules! define_message {
    ($struct_name:ident, $message_type_name:expr, $message_id:expr) => {
        pub struct $struct_name {
            message: $crate::message::piranha_message::PiranhaMessage,
            client: std::sync::Arc<tokio::sync::Mutex<crate::networking::client::ClientConnection>>,
        }

        impl $struct_name {
            pub fn new(
                bytes: bytes::BytesMut,
                client: std::sync::Arc<tokio::sync::Mutex<crate::networking::client::ClientConnection>>,
            ) -> Self {
                let mut message = $crate::message::piranha_message::PiranhaMessage::new(
                    bytes,
                    client.clone(),
                    $message_type_name,
                );
                message.set_message_id($message_id);
                $struct_name { message, client }
            }

            pub fn get_message_type_name() -> &'static str {
                $message_type_name
            }
        }

        impl $crate::message::piranha_message::MessageOps for $struct_name {
            fn get_stream(&mut self) -> &mut $crate::stream::byte_stream::ByteStream {
                self.message.get_stream()
            }

            fn get_client(
                &self,
            ) -> std::sync::Arc<tokio::sync::Mutex<crate::networking::client::ClientConnection>> {
                self.message.get_client()
            }

            fn get_message_id(&self) -> u16 {
                self.message.get_message_id()
            }

            fn set_message_id(&mut self, id: u16) {
                self.message.set_message_id(id)
            }

            fn get_message_version(&self) -> u16 {
                self.message.get_message_version()
            }

            fn set_message_version(&mut self, version: u16) {
                self.message.set_message_version(version)
            }

            fn get_message_type_name(&self) -> &str {
                self.message.get_message_type_name()
            }
        }

        impl $crate::message::piranha_message::FactoryMessage for $struct_name {
            fn new(
                buffer: bytes::BytesMut,
                client: std::sync::Arc<tokio::sync::Mutex<crate::networking::client::ClientConnection>>,
            ) -> Self {
                Self::new(buffer, client)
            }
        }
    };
}