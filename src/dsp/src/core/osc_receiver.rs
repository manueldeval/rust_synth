use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use rosc::{OscMessage, OscPacket};
use std::thread;
use std::{
    net::{SocketAddrV4, UdpSocket},
    str::FromStr,
};
use thiserror::Error;

pub trait OscMessageHandler {
    fn handle_message(&mut self, message: OscMessage);
}

pub enum OscReceiverCommand {
    Stop,
}

#[derive(Error, Debug)]
pub enum OscReceiverError {
    #[error("Not a valid UDP address: `{0}`")]
    NotAValidAddress(String),
}

pub struct OscReceiver<T: OscMessageHandler> {
    command_sender: Sender<OscReceiverCommand>,
    command_receiver: Receiver<OscReceiverCommand>,
    osc_address: String,
    osc_message_handler: T,
}

impl<T> OscReceiver<T>
where
    T: OscMessageHandler + Send + Clone + 'static,
{
    pub fn new(osc_address: String, osc_message_handler: T) -> Self {
        let (command_sender, command_receiver) = unbounded();
        Self {
            command_sender,
            command_receiver,
            osc_address,
            osc_message_handler,
        }
    }

    pub fn start(&mut self) -> Result<(), OscReceiverError> {
        let command_receiver = self.command_receiver.clone();
        let addr = SocketAddrV4::from_str(&self.osc_address)
            .map_err(|_| OscReceiverError::NotAValidAddress(self.osc_address.to_owned()))?;
        let mut message_handler = self.osc_message_handler.clone();
        thread::spawn(move || {
            let sock = UdpSocket::bind(addr).unwrap();
            println!("Listening osc message on UDP address: {}", addr);
            let mut buf = [0u8; 1536];

            loop {
                match command_receiver.try_recv() {
                    // No command message
                    Err(TryRecvError::Empty) => {
                        match sock.recv_from(&mut buf) {
                            Ok((size, addr)) => {
                                println!("Received packet with size {} from: {}", size, addr);
                                let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                                match packet {
                                    OscPacket::Message(msg) => {
                                        message_handler.handle_message(msg);

                                        // match (&msg.addr[..],msg.args.as_slice()) {
                                        //     ("/button/",[OscType::Float(e)]) => {
                                        //         println!("BUTTON: {}",e)
                                        //     }
                                        //     _ => {
                                        //         println!("No match for OSC address: {}, OSC arguments: {:?}", msg.addr,msg.args);
                                        //     }
                                        // };
                                    }
                                    OscPacket::Bundle(bundle) => {
                                        println!("OSC Bundle: {:?}", bundle);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error receiving from socket: {}", e);
                                break;
                            }
                        }
                    }
                    // Stop
                    Ok(OscReceiverCommand::Stop) => {
                        break;
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    pub fn stop(&mut self) {
        let _ = self.command_sender.send(OscReceiverCommand::Stop);
    }
}
