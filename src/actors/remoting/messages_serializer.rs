use crate::actors::message::Message;
use std::fmt;

pub struct SerializedMessage {
    pub marker: u32,
    pub blob: Vec<u8>
}

impl fmt::Display for SerializedMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SerializedMessage [ marker={:?}, blob={:X?} ]", self.marker, self.blob)
    }
}

#[derive(Debug)]
pub enum SerializationError {
    UnsupportedMessage,
    DamagedBlob
}

pub trait MessagesSerializer {
    fn to_binary(&mut self, msg: Message) -> Result<SerializedMessage, SerializationError>;
    fn from_binary(&mut self, marker: u32, blob: Vec<u8>) -> Result<Message, SerializationError>;
}