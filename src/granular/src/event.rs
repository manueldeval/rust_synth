use crossbeam::channel::{Receiver, Sender, unbounded};
use strum_macros::Display;

#[derive(Display)]
pub enum Event {
    // Channel/note/velocity
    NoteOn(u8,u8,u8),
    // Channel/note
    NoteOff(u8,u8),
}

#[derive(Clone)]
pub struct EventSender {
    sender: Sender<Event>
}

impl EventSender {
    pub fn send(&self, event: Event) {
        self.sender.send(event).expect("Unable to send message");
    }
}

#[derive(Clone)]
pub struct EventReceiver {
    receiver: Receiver<Event>
}

impl EventReceiver {
    pub fn receive(&self) -> Option<Event> {
        self.receiver
            .try_recv()
            .map(|e| Some(e))
            .unwrap_or(None)
    }
}

pub fn create_event_sender_receiver() -> (EventSender,EventReceiver) {
    let (s,r) = unbounded();
    (EventSender {sender: s}, EventReceiver{receiver: r})
}
