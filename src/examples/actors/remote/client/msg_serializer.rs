// This struct describes how to serialize and deserialize messages. For simplification reasons,
// serde_json is used for generating and parsing binary data. This struct represents client
// side serializer, he may serialize SendData message and deserialize Data message.

use crate::actors::prelude::*;
use super::remote_messages::storage;
use bytes::{BytesMut, Bytes, Buf, BufMut, BigEndian};
use std::sync::{Arc, Mutex};

pub struct MsgSerializer {}

impl MsgSerializer {
    pub fn new() -> MsgSerializer {
        MsgSerializer {}
    }
}

impl MessagesSerializer for MsgSerializer {
    fn to_binary(&mut self, msg: Message) -> Result<SerializedMessage, SerializationError> {
        let msg = msg.get();
        match_downcast_ref!(msg, {
            m: storage::commands::GetData => {
                let s = serde_json::to_string(&m).unwrap();
                let mut buf = Vec::new();
                buf.put(s.as_bytes());
                Ok(SerializedMessage { marker: 1, blob: buf })
            },
            _ => Err(SerializationError::UnsupportedMessage)
        })
    }

    fn from_binary(&mut self, marker: u32, blob: Vec<u8>) -> Result<Message, SerializationError> {
        match marker {
            2 => {
                let string = std::str::from_utf8(&blob);
                if string.is_ok() {
                    let m: Result<storage::responses::Data, serde_json::Error> = serde_json::from_str(string.unwrap());
                    if m.is_ok() {
                        Ok(msg!(m.ok().unwrap()))
                    } else {
                        Err(SerializationError::DamagedBlob)
                    }
                } else {
                    Err(SerializationError::DamagedBlob)
                }
            },
            _ => Err(SerializationError::UnsupportedMessage)
        }
    }
}