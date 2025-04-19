use crate::message::piranha_message::{Coder, MessageOps};
use crate::define_message;



define_message!(
    AuthenticationResponseMessage,
    "AuthenticationResponseMessage",
    20104
);
impl Coder for AuthenticationResponseMessage {
      async fn encode(&mut self) -> &mut Self {
        self.write_int(self.client.lock().await.get_high_id().await);
        self.write_int(self.client.lock().await.get_low_id().await);

        self.write_int(self.client.lock().await.get_high_id().await);
        self.write_int(self.client.lock().await.get_low_id().await);

        self.write_string(Option::from("")); // id

        self.write_string(Option::from("")); // facebook
        self.write_string(Option::from("")); // game center

        self.write_int(29);
        self.write_int(0);
        self.write_int(0);

        self.write_string(Option::from("dev"));

        self.write_int(0);
        self.write_int(0);
        self.write_int(0);

        self.write_string(Option::from(""));
        self.write_string(Option::from(""));
        self.write_string(Option::from(""));

        self.write_int(0);

        self.write_string(Option::from(""));
        self.write_string(Option::from("RU")); // region
        self.write_string(Option::from(""));

        self.write_int(1);

        self.write_string(Option::from(""));

        self.write_int(0);

        self.write_int(0);

        self.write_vint(0);

        self.write_string(Option::from(""));

        self.write_vint(1);
        self.write_vint(1);

        self.write_string(Option::from(""));
        self
    }

    async fn decode(&mut self) -> &mut Self {
        self
    }

    async fn process(&mut self) -> &mut Self {
        self
    }
}
