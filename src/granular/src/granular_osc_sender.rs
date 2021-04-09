use rosc::{encoder, OscType};
use std::{
    net::{SocketAddrV4, UdpSocket},
    str::FromStr,
};

use crate::{GuiEvent, GuiEventReceiver};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use rosc::{OscMessage, OscPacket};
use std::thread;
use thiserror::Error;

enum GranularOscMessageSenderCommand {
    Stop,
}

#[derive(Error, Debug)]
pub enum GranularOscSenderError {
    #[error("Not a valid UDP address: `{0}`")]
    NotAValidAddress(String),
}

pub struct GranularOscMessageSender {
    gui_event_receiver: GuiEventReceiver,
    command_sender: Sender<GranularOscMessageSenderCommand>,
    command_receiver: Receiver<GranularOscMessageSenderCommand>,
    local_osc_address: String,
    remote_osc_address: String,
}

impl GranularOscMessageSender {
    pub fn new(
        local_osc_address: String,
        remote_osc_address: String,
        gui_event_receiver: &GuiEventReceiver,
    ) -> Self {
        let (command_sender, command_receiver) = unbounded();
        Self {
            command_sender,
            command_receiver,
            local_osc_address,
            remote_osc_address,
            gui_event_receiver: gui_event_receiver.clone(),
        }
    }

    pub fn start(&mut self) -> Result<(), GranularOscSenderError> {
        let command_receiver = self.command_receiver.clone();
        let local_addr = SocketAddrV4::from_str(&self.local_osc_address).map_err(|_| {
            GranularOscSenderError::NotAValidAddress(self.local_osc_address.to_owned())
        })?;
        let remote_addr = SocketAddrV4::from_str(&self.remote_osc_address).map_err(|_| {
            GranularOscSenderError::NotAValidAddress(self.remote_osc_address.to_owned())
        })?;
        let gui_recv = self.gui_event_receiver.clone();

        thread::spawn(move || {
            let sock = UdpSocket::bind(local_addr).unwrap();
            println!("Listening osc message on UDP address: {}", local_addr);

            loop {
                // match command_receiver.try_recv() {
                //     // No command message
                //     Err(TryRecvError::Empty) => {
                //         if let Some(e) = gui_recv.receive() {
                //             match e {
                //                 GuiEvent::SampleRms(data) => {
                //                     let converted: Vec<String> = data
                //                         .iter()
                //                         .enumerate()
                //                         .map(|(i, v)| format!("[{},{}]", i, v))
                //                         .collect();
                //                     let payload = format!("[{}]", converted.join(","));
                //                     let msg_buf =
                //                         encoder::encode(&OscPacket::Message(OscMessage {
                //                             addr: "/rms".to_string(),
                //                             args: vec![OscType::String(payload)],
                //                         }))
                //                         .unwrap();

                //                     sock.send_to(&msg_buf, remote_addr).unwrap();
                //                 }
                //                 GuiEvent::Position(pos) => {
                //                     let msg_buf =
                //                         encoder::encode(&OscPacket::Message(OscMessage {
                //                             addr: "/position".to_string(),
                //                             args: vec![OscType::Float(pos)],
                //                         }))
                //                         .unwrap();
                //                     sock.send_to(&msg_buf, remote_addr).unwrap();
                //                 }
                //             };
                //         }
                //     }
                //     // Stop
                //     Ok(GranularOscMessageSenderCommand::Stop) => {
                //         break;
                //     }
                //     _ => {}
                // }
                match command_receiver.try_recv() {
                    // No command message
                    Err(TryRecvError::Empty) => {
                        if let Some(e) = gui_recv.receive() {
                            match e {
                                GuiEvent::SampleRms(data) => {
                                    let converted: Vec<String> = data
                                        .iter()
                                        .enumerate()
                                        .map(|(i, v)| format!("[{},{}]", i, v))
                                        .collect();
                                    let payload = format!("[{}]", converted.join(","));
                                    let msg_buf =
                                        encoder::encode(&OscPacket::Message(OscMessage {
                                            addr: "/rms".to_string(),
                                            args: vec![OscType::String(payload)],
                                        }))
                                        .unwrap();

                                    sock.send_to(&msg_buf, remote_addr).unwrap();
                                }
                                GuiEvent::Position(pos) => {
                                    let msg_buf =
                                        encoder::encode(&OscPacket::Message(OscMessage {
                                            addr: "/position".to_string(),
                                            args: vec![OscType::Float(pos)],
                                        }))
                                        .unwrap();
                                    sock.send_to(&msg_buf, remote_addr).unwrap();
                                }
                            };
                        }
                    }
                    // Stop
                    Ok(GranularOscMessageSenderCommand::Stop) => {
                        break;
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    pub fn stop(&mut self) {
        let _ = self
            .command_sender
            .send(GranularOscMessageSenderCommand::Stop);
    }
}
