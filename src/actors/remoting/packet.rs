//! Representation of the library's network packet

use bytes::{BytesMut, Bytes, Buf, BufMut, BigEndian};
use std::fmt;
use std::io::{Cursor, SeekFrom, Seek};

pub static PREAMBLE: u64 = 0x1f2f3f3f4f6f7f8f;

#[derive(Debug)]
pub enum Opm {
    ActorSelect,
    ActorSelectResponse,
    SendMsg
}


pub struct Packet {
    pub id: u32,
    pub opm: Opm,
    pub length: u16,
    pub body: Vec<u8>

}

impl Packet {
    pub fn new(id: u32, opm: u8, length: u16, body: Vec<u8>) -> Packet {
        Packet { id, opm: Self::code_to_opm(opm), length, body }
    }

    pub fn new_actor_of(id: u32, path: &str) -> Packet {
        let path = path.as_bytes();
        let size = path.len();
        //let path = path;

        let mut buf = vec![];
        buf.put(&path[..]);

        Packet {
            id,
            opm:  Self::code_to_opm(1),
            length: size as u16,
            body: buf
        }
    }

    pub fn new_actor_of_response(id: u32, ids: Vec<u32>) -> Packet {
        let size = ids.len() * 4;

        let mut buf = vec![];
        for id in ids {
            buf.put_u32_be(id);
        }

        Packet {
            id,
            opm: Self::code_to_opm(2),
            length: size as u16,
            body: buf
        }
    }

    pub fn new_send_msg(id: u32, marker: u32, blob: &[u8], rarid: u32, larid: u32) -> Packet {
        let mut buf = Vec::new();
        buf.put_u32_be(marker);
        buf.put_u32_be(rarid);
        buf.put_u32_be(larid);
        buf.put(&blob[..]);



        Packet {
            id,
            opm: Self::code_to_opm(3),
            length: buf.len() as u16,
            body: buf
        }
    }

    pub fn body_as_actor_of(&self) -> String {
        String::from(std::str::from_utf8(&self.body).unwrap())
    }

    pub fn body_as_send_msg(&self) -> (u32, u32, u32, Vec<u8>) {
        let len = self.body.len();
        let mut buf = Cursor::new(vec![0; len]);
        buf.put(&self.body);

        buf.seek(SeekFrom::Start(0));
        let marker = buf.get_u32_be();
        buf.seek(SeekFrom::Start(4));
        let larid = buf.get_u32_be();
        buf.seek(SeekFrom::Start(8));
        let rarid = buf.get_u32_be();
        let blob = Vec::from(&self.body[12..len]);


        (marker, larid, rarid, blob)
    }

    pub fn body_as_actor_of_response(&self) -> Vec<u32> {
        let len =  self.body.len();
        let count =  len / 4;
        let counter = 0;
        let mut refs = Vec::new();

        let mut buf = Cursor::new(vec![0; len]);

        buf.put(&self.body);

        for i in 0..count {
            buf.seek(SeekFrom::Start(counter * 4));
            refs.push(buf.get_u32_be());
        }

        refs
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let size = (15 + self.length) as usize;
        let mut buf = vec![];

        buf.put_u64_be(PREAMBLE);
        buf.put_u32_be(self.id);
        buf.put_u8(Self::opm_to_code(&self.opm));
        buf.put_u16_be(self.length);
        buf.put(self.body.to_vec());

        buf.to_vec()
    }

    fn code_to_opm(code: u8) -> Opm {
        match code {
            1 => Opm::ActorSelect,
            2 => Opm::ActorSelectResponse,
            3 => Opm::SendMsg,
            e => panic!("Unexpected opm code '{}'", e)
        }
    }

    fn opm_to_code(opm: &Opm) -> u8 {
        match *opm {
            Opm::ActorSelect => 1,
            Opm::ActorSelectResponse => 2,
            Opm::SendMsg => 3
        }
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Packet [ id={}, opm={:?}, length={}, body={:X?} ]", self.id, self.opm, self.length, self.body)
    }
}