use crate::define_message;
use crate::message::piranha_message::{Coder, MessageOps};


define_message!(
    ClientHelloMessage,
    "ClientHelloMessage",
    10100
);


impl Coder for ClientHelloMessage {
    async fn encode(&mut self) -> &mut Self { self }
    async fn decode(&mut self) -> &mut Self { self }
    async fn process(&mut self) -> &mut Self { self }

}