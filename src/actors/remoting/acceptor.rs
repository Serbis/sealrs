use super::connection::{ConnectionData, ServerConnection};
use super::packet::Packet;
use crate::common::tsafe::TSafe;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

#[derive(Clone)]
pub struct Acceptor {
    stop_acceptor: TSafe<bool>,
    stop_accepting: TSafe<bool>
}

impl Acceptor {
    pub fn new(addr: SocketAddr, sender: Sender<(ServerConnection, Receiver<ConnectionData>)>) -> Acceptor {
        let stop_acceptor_o = tsafe!(false);
        let stop_accepting_o = tsafe!(false);
        let stop_acceptor = stop_acceptor_o.clone();
        let stop_accepting = stop_accepting_o.clone();


        thread::spawn(move || {

            while !*stop_acceptor.lock().unwrap() {
                let listener = TcpListener::bind(addr);

                if listener.is_ok() {
                    let listener = listener.ok().unwrap();
                    listener.set_nonblocking(true);
                    trace!("Remote acceptor is successfully bound [ addr={} ]", addr);

                    while !*stop_accepting.lock().unwrap() {
                        let stream = listener.accept();

                        if stream.is_ok() {
                            let (stream, _) = stream.unwrap();
                            let (s, r) = mpsc::channel();
                            let connection = ServerConnection::new(stream, s);
                            sender.send((connection, r));
                            trace!("Accepted new connection [ addr={} ]", addr)
                        } else {
                            let err = stream.err().unwrap();

                            match err.kind() {
                                ErrorKind::WouldBlock => {
                                    //FIXME this is a vary bad solution!
                                    thread::sleep_ms(500);
                                    continue
                                },
                                _ => {
                                    trace!("Accept operation failed [ addr={}, err={} ]", addr, err);
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    error!("Unable to bound acceptor [ addr={} ]", addr);
                }

                thread::sleep(Duration::from_secs(3));
            }

            trace!("Server acceptor was stopped")
        });

        Acceptor {
            stop_acceptor: stop_acceptor_o,
            stop_accepting: stop_accepting_o
        }
    }

    pub fn stop(&mut self) {
        *self.stop_acceptor.lock().unwrap() = true;
        *self.stop_accepting.lock().unwrap() = true;
    }
}