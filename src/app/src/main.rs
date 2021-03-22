use std::io::{stdin, stdout, Write};
extern crate termion;

use termion::{event::Key, input::TermRead};
use termion::raw::IntoRawMode;
extern crate anyhow;

extern crate audio;

use audio::{player::Player, synthengine::StereoGeneratorFactory};
use granular::{Granular, event::{Event, EventReceiver, EventSender, create_event_sender_receiver}};

struct SoundGeneratorFactory {
    pub event_sender: EventSender,
    pub event_receiver: EventReceiver,
}
impl Clone for SoundGeneratorFactory {
    fn clone(&self) -> Self {
        SoundGeneratorFactory {
            event_sender: self.event_sender.clone(),
            event_receiver: self.event_receiver.clone()
        }
    }
}

impl SoundGeneratorFactory {
    pub fn new() -> Self {
        let (event_sender,event_receiver) = create_event_sender_receiver();
        SoundGeneratorFactory {
            event_sender,
            event_receiver
        }
    }
}
impl StereoGeneratorFactory for SoundGeneratorFactory {
    type Gen = Granular;
    fn create(&self) -> Self::Gen {
        let mut osc = Granular::new(self.event_receiver.clone());
        osc.set_frequency(220.0);
        osc
    }
}

fn main() -> anyhow::Result<()> {
    let synth_factory = SoundGeneratorFactory::new();
    let event_sender = synth_factory.event_sender.clone();
    let mut player = Player::new(synth_factory, None);
    player.start()?;
    event_loop(event_sender);
    player.stop();
    Ok(())
}

fn event_loop(sender: EventSender) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    for c in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::All
        )
        .unwrap();
        match c.unwrap() {
            Key::Char('a') => sender.send(Event::NoteOn(1,57,127)),
            Key::Char('z') => sender.send(Event::NoteOn(1,58,127)),
            Key::Char('e') => sender.send(Event::NoteOn(1,59,127)),
           
            Key::Ctrl('a') => sender.send(Event::NoteOff(1,57)),
            Key::Ctrl('z') => sender.send(Event::NoteOff(1,58)),
            Key::Ctrl('e') => sender.send(Event::NoteOff(1,59)),

            Key::Char('q') | Key::Ctrl('c') => {
                println!("Quit");
                break
            },
            Key::Char(c) => {println!("{}",c);},
            _ => {},
        }
        stdout.flush().unwrap();
    }

}
