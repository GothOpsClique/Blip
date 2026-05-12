include!(concat!(env!("OUT_DIR"), "/blip.rs"));

use prost::Message as ProstMessage;

pub fn encode_message(message: &ChatMessage) -> Vec<u8> {
    let mut buffer = Vec::new();
    message
        .encode(&mut buffer)
        .expect("failed to encode protobuf message");
    buffer
}

pub fn decode_message(bytes: &[u8]) -> Result<ChatMessage, prost::DecodeError> {
    ChatMessage::decode(bytes)
}
