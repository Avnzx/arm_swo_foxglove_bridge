use foxglove::{Encode, Schema};
use prost::{EncodeError, bytes::BufMut};

#[derive(Clone, PartialEq, prost::Message)]
pub struct NumericalMessage {
    #[prost(message, optional, tag = "1")]
    pub timestamp: Option<foxglove::schemas::Timestamp>,
    #[prost(double, tag = "2")]
    pub number: f64,
}

impl foxglove::Encode for NumericalMessage {
    type Error = EncodeError;

    fn get_schema() -> std::option::Option<foxglove::Schema> {
        Some(Schema::new(
            "bluesat.NumericalMessage",
            "protobuf",
            include_bytes!("NumericalMessage.fds"),
        ))
    }

    fn get_message_encoding() -> std::string::String {
        "protobuf".to_string()
    }

    fn encode(&self, buf: &mut impl BufMut) -> Result<(), <Self as Encode>::Error> {
        prost::Message::encode(self, buf)
    }
}
