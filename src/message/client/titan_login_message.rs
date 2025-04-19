use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{define_message, log};
use crate::message::piranha_message::{Coder, MessageOps, PiranhaMessage};
use crate::networking::client::ClientConnection;

define_message!(
    TitanLoginMessage,
    "TitanLoginMessage",
    10101
);




impl Coder for TitanLoginMessage   {

    async fn encode(&mut self) -> &mut Self {
        let high_id = self.read_int();
        let low_id = self.read_int();

        self.client.lock().await.set_high_id(high_id);
        self.client.lock().await.set_low_id(low_id);

        self
    }

    async fn decode(&mut self) -> &mut Self {
        self
    }

    async fn process(&mut self) -> &mut Self {
        log!(&format!("high id: {} | low id: {}",
            self.client.lock().await.get_high_id(),
            self.client.lock().await.get_low_id()
        ));
        self
    }
}

