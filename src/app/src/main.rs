use std::io::prelude::*;
use std::{
    io,
    sync::{Arc, Mutex},
};

use audio::{player::Player, synthengine::StereoGeneratorFactory};
use dsp::core::OscReceiver;
use granular::{
    create_event_sender_receiver, EventReceiver, EventSender, GranularController,
    GranularOscMessageHandler, GranularSynth,
};

//===================================
// Synth factory
#[derive(Clone)]
struct SoundGeneratorFactory {
    event_sender: EventSender,
    event_receiver: EventReceiver,
}
impl SoundGeneratorFactory {
    pub fn new() -> Self {
        let (s, r) = create_event_sender_receiver();
        SoundGeneratorFactory {
            event_sender: s,
            event_receiver: r,
        }
    }
}

impl StereoGeneratorFactory for SoundGeneratorFactory {
    type Gen = GranularSynth;
    fn create(&self) -> Self::Gen {
        GranularSynth::new(self.event_receiver.clone())
    }
}

fn main() {
    // Sound generator
    let sound_generator_factory = SoundGeneratorFactory::new();
    let control_channel = sound_generator_factory.event_sender.clone();
    let mut player = Player::new(sound_generator_factory, None);

    let sample_rate = player.start().unwrap();

    let mut synth_controller = GranularController::new(control_channel, sample_rate);
    let _ = synth_controller
       .load_samples("/home/deman/projets/perso/rust/granular/mission24000.wav".to_owned());

    let wrapped_synth_controller = Arc::new(Mutex::new(synth_controller));
    let mut osc_receiver = OscReceiver::new(
        "127.0.0.1:9666".to_owned(),
        GranularOscMessageHandler::new(wrapped_synth_controller.clone()),
    );
    osc_receiver.start().unwrap();
    pause();
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
