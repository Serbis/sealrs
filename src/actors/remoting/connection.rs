//! Server and client side connections.
//!
//! This is the border point between physical tcp network and abstraction of actor's. Connections
//! works with a binary data from tcp streams. They parse packets from the network and sends new
//! binary packets to her.

use super::packet::Packet;
use super::packet::PREAMBLE;
use crate::common::tsafe::TSafe;
use arraydeque::{ArrayDeque, Array};
use bytes::{BytesMut, Bytes, Buf, BufMut, BigEndian};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::prelude::*;
use std::time::Duration;
use std::io::{Cursor, SeekFrom, ErrorKind};

enum ParseMode {
    Preamble,
    Header,
    Body
}

pub enum ConnectionData {
    ReceivedPacket(Packet),
    Closed
}

pub struct ClientConnection {
    stream: TSafe<Option<TcpStream>>,
    stop_stream: TSafe<bool>,
    stop_conn: TSafe<bool>
}

impl ClientConnection {
    pub fn new(addr: SocketAddr, sender: Sender<ConnectionData>) -> ClientConnection {
        let stream = tsafe!(None);
        let stop_stream = tsafe!(false);
        let stop_conn = tsafe!(false);

        Self::serve(addr, stream.clone(), stop_stream.clone(), stop_conn.clone(), sender);

        ClientConnection {
            stream,
            stop_stream,
            stop_conn
        }
    }


    pub fn serve(addr: SocketAddr, own_stream: TSafe<Option<TcpStream>>, stop_stream: TSafe<bool>, stop_conn: TSafe<bool>, sender: Sender<ConnectionData>) {

        thread::spawn(move || {

            while !*stop_conn.lock().unwrap() {
                let mut buffer = [0; 1];

                // Утановкить соединение

                let stream = TcpStream::connect(addr.clone());
                if stream.is_ok() {
                    trace!("Connection was success established [ addr={} ]", addr);
                    *stop_stream.lock().unwrap() = false;

                    let mut stream = stream.ok().unwrap();
                    stream.set_read_timeout(Some(Duration::from_secs(1)));
                    {
                        let mut own_stream = own_stream.lock().unwrap();
                        *own_stream = Some(stream.try_clone().unwrap());
                    }


                    let mut in_buf: ArrayDeque<[u8; 32768]> = ArrayDeque::new();
                    let mut mode = ParseMode::Preamble;
                    let mut id = 0;
                    let mut opm = 0;
                    let mut length = 0;

                    while !*stop_stream.lock().unwrap() {
                        let ch = stream.read_exact(&mut buffer[..]);

                        if ch.is_ok() {
                            let ch = buffer[0];
                            //println!("{:?}", ch);
                            in_buf.push_back(ch);
                            let dlen = in_buf.len();

                            match mode {
                                ParseMode::Preamble => {
                                    if dlen >= 8 {
                                        let mut buf = Cursor::new(vec![0; 8]);
                                        for i in 0..8 {
                                            buf.put_u8(*in_buf.get(i).unwrap());
                                        }

                                        buf.seek(SeekFrom::Start(0));

                                        if buf.get_u64_be() == PREAMBLE {
                                            in_buf.clear();
                                            mode = ParseMode::Header;
                                        }
                                    }
                                },
                                ParseMode::Header => {
                                    if dlen >= 7 {
                                        let mut buf = Cursor::new(vec![0; 7]);
                                        for i in 0..7 {
                                            buf.put_u8(*in_buf.get(i).unwrap());
                                        }

                                        buf.seek(SeekFrom::Start(0));
                                        id = buf.get_u32_be();
                                        buf.seek(SeekFrom::Start(4));
                                        opm = buf.get_u8();
                                        buf.seek(SeekFrom::Start(5));
                                        length = buf.get_u16_be();

                                        in_buf.clear();
                                        mode = ParseMode::Body;
                                    }
                                },
                                ParseMode::Body => {
                                    if dlen >= length as usize {
                                        let mut buf = vec![];

                                        for i in 0..length {
                                            buf.put_u8(*in_buf.get(i as usize).unwrap());
                                        }

                                        let packet = Packet::new(id, opm, length, buf);
                                        sender.send(ConnectionData::ReceivedPacket(packet));
                                        in_buf.clear();
                                        mode = ParseMode::Preamble;
                                    }
                                }
                            }

                        } else {
                            let err = ch.err().unwrap().kind();

                            match err {
                                ErrorKind::WouldBlock => continue,
                                _ => {
                                    error!("IO error while reading stream [ addr={}, err={:?} ]", addr, err);

                                    break;
                                }
                            }
                        }
                    }

                    trace!("Stream was break [ addr={} ]", addr)
                } else {
                    let mut own_stream = own_stream.lock().unwrap();
                    *own_stream = None;
                    error!("Unable to establish remote connection [ addr={} ]", addr)
                }

                thread::sleep(Duration::from_secs(3))

            }

            sender.send(ConnectionData::Closed);
            trace!("Connection was closed")
        });
    }


    pub fn send(&mut self, packet: Packet) -> bool {
        let stream = self.stream.lock().unwrap();
        if stream.is_some() {
            let result = stream.as_ref().unwrap().write(&packet.to_binary()[..]);
            if result.is_err() {
                *self.stop_stream.lock().unwrap() = true;
                error!("Unable write data to the stream. Stream is broken");
                return false
            }
        } else {
            error!("Unable write data to the stream. Stream in inconsistent state");
            return false
        }

        true
    }

    pub fn close(&mut self) {
        let mut stop_stream = self.stop_stream.lock().unwrap();
        *stop_stream = true;

        let mut stop_conn = self.stop_conn.lock().unwrap();
        *stop_conn = true;

        //*self.stream.lock().unwrap() = None;
        //println!("XXX");
    }
}

impl Drop for ClientConnection {
    fn drop(&mut self) {
        println!("ClientConnection dropped")
    }
}

//=================================================================================================

pub struct ServerConnection {
    stream: TcpStream,
    stop_stream: TSafe<bool>
}

impl ServerConnection {
    pub fn new(stream: TcpStream, sender: Sender<ConnectionData>) -> ServerConnection {
        let stop_stream = tsafe!(false);

        Self::serve(stream.try_clone().unwrap(), stop_stream.clone(), sender);

        ServerConnection {
            stream,
            stop_stream
        }
    }


    pub fn serve(stream: TcpStream, stop_stream: TSafe<bool>, sender: Sender<ConnectionData>) {

        thread::spawn(move || {
            let mut buffer = [0; 1];

            let mut stream = stream;
            stream.set_read_timeout(Some(Duration::from_secs(1)));

            let mut in_buf: ArrayDeque<[u8; 32768]> = ArrayDeque::new();
            let mut mode = ParseMode::Preamble;
            let mut id = 0;
            let mut opm = 0;
            let mut length = 0;

            while !*stop_stream.lock().unwrap() {

                let ch = stream.read_exact(&mut buffer[..]);

                if ch.is_ok() {
                    let ch = buffer[0];
                    //println!("{:?}", ch);
                    in_buf.push_back(ch);
                    let dlen = in_buf.len();

                    match mode {
                        ParseMode::Preamble => {
                            if dlen >= 8 {
                                let mut buf = Cursor::new(vec![0; 8]);
                                for i in 0..8 {
                                    buf.put_u8(*in_buf.get(i).unwrap());
                                }

                                buf.seek(SeekFrom::Start(0));

                                if buf.get_u64_be() == PREAMBLE {
                                    in_buf.clear();
                                    mode = ParseMode::Header;
                                }
                            }
                        },
                        ParseMode::Header => {
                            if dlen >= 7 {
                                let mut buf = Cursor::new(vec![0; 7]);
                                for i in 0..7 {
                                    buf.put_u8(*in_buf.get(i).unwrap());
                                }

                                buf.seek(SeekFrom::Start(0));
                                id = buf.get_u32_be();
                                buf.seek(SeekFrom::Start(4));
                                opm = buf.get_u8();
                                buf.seek(SeekFrom::Start(5));
                                length = buf.get_u16_be();

                                in_buf.clear();
                                mode = ParseMode::Body;
                            }
                        },
                        ParseMode::Body => {
                            if dlen >= length as usize {
                                let mut buf = vec![];

                                for i in 0..length {
                                    buf.put_u8(*in_buf.get(i as usize).unwrap());
                                }

                                let packet = Packet::new(id, opm, length, buf);
                                sender.send(ConnectionData::ReceivedPacket(packet));
                                in_buf.clear();
                                mode = ParseMode::Preamble;
                            }
                        }
                    }

                } else {
                    let err = ch.err().unwrap().kind();

                    match err {
                        ErrorKind::WouldBlock => continue,
                        _ => {
                            error!("IO error while reading stream [ err={:?} ]", err);

                            break;
                        }
                    }
                }
            }

            sender.send(ConnectionData::Closed);
            trace!("Stream was break. Connection closed")
        });
    }

    pub fn receive(&mut self) -> Packet {
        unimplemented!()
    }

    pub fn send(&mut self, packet: Packet) -> bool {
        let result = self.stream.write(&packet.to_binary()[..]);
        if result.is_err() {
            *self.stop_stream.lock().unwrap() = true;
            error!("Unable write data to the stream. Stream is broken");
            return false;
        }

        true
    }

    pub fn close(&mut self) {
        let mut stop_stream = self.stop_stream.lock().unwrap();
        *stop_stream = true;
    }
}

impl Drop for ServerConnection {
    fn drop(&mut self) {
        println!("ServerConnection dropped")
    }
}
