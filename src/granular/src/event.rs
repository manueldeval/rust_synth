use std::time::Duration;

use crossbeam::channel::{unbounded, Receiver, Sender};
use strum_macros::Display;

#[derive(Display)]
pub enum SynthEvent {
    LoadSound(Vec<f32>),
    MainLevel(f32),
    Start(usize),
    End(usize),
    Step(f32),
    PanSpread(f32),
    ScanSpread(f32),
    GrainStep(f32),
    GrainsPerSec(f32),
    GrainEnvelop(f32,f32,f32),
}

#[derive(Clone)]
pub struct SynthEventSender {
    sender: Sender<SynthEvent>,
}

impl SynthEventSender {
    pub fn send(&self, event: SynthEvent) {
        self.sender.send(event).expect("Unable to send message");
    }
}

#[derive(Clone)]
pub struct SynthEventReceiver {
    receiver: Receiver<SynthEvent>,
}

impl SynthEventReceiver {
    pub fn receive(&self) -> Option<SynthEvent> {
        self.receiver.try_recv().map(|e| Some(e)).unwrap_or(None)
    }
}

pub fn create_synth_event_sender_receiver() -> (SynthEventSender, SynthEventReceiver) {
    let (s, r) = unbounded();
    (
        SynthEventSender { sender: s },
        SynthEventReceiver { receiver: r },
    )
}

#[derive(Display)]
pub enum GuiEvent {
    SampleRms(Vec<f32>),
    Position(f32),
}

#[derive(Clone)]
pub struct GuiEventSender {
    sender: Sender<GuiEvent>,
}

impl GuiEventSender {
    pub fn send(&self, event: GuiEvent) {
        self.sender.send(event).expect("Unable to send message");
    }
}

#[derive(Clone)]
pub struct GuiEventReceiver {
    receiver: Receiver<GuiEvent>,
}

impl GuiEventReceiver {
    pub fn receive(&self) -> Option<GuiEvent> {
        self.receiver.recv_timeout(Duration::from_millis(100)).map(|e| Some(e)).unwrap_or(None)
    }
}

pub fn create_gui_event_sender_receiver() -> (GuiEventSender, GuiEventReceiver) {
    let (s, r) = unbounded();
    (
        GuiEventSender { sender: s },
        GuiEventReceiver { receiver: r },
    )
}
