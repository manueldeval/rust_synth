use audio::{player::Player, synthengine::StereoGeneratorFactory};
use dsp::core::OscReceiver;
use granular::{
    create_gui_event_sender_receiver, create_synth_event_sender_receiver,
    synth_to_ctrl_state_synchro, GranularController, GranularOscMessageHandler,
    GranularOscMessageSender, GranularSynth, SynthEventReceiver, SynthEventSender, SynthState,
};
use ring_channel::{ring_channel, RingReceiver, RingSender};
use std::io::prelude::*;
use std::num::NonZeroUsize;
use std::{
    io,
    sync::{Arc, Mutex},
};

//===================================
// Synth factory
#[derive(Clone)]
struct SoundGeneratorFactory {
    event_sender: SynthEventSender,
    event_receiver: SynthEventReceiver,
    state_sender: RingSender<SynthState>,
    state_receiver: RingReceiver<SynthState>,
}
impl SoundGeneratorFactory {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = create_synth_event_sender_receiver();
        let (state_sender, state_receiver) = ring_channel(NonZeroUsize::new(100).unwrap());

        SoundGeneratorFactory {
            event_sender,
            event_receiver,
            state_sender,
            state_receiver,
        }
    }
}

impl StereoGeneratorFactory for SoundGeneratorFactory {
    type Gen = GranularSynth;
    fn create(&self) -> Self::Gen {
        GranularSynth::new(self.event_receiver.clone(), self.state_sender.clone())
    }
}

pub struct OscSender {}

fn main() {
    // Sound generator
    let (gui_send, gui_recv) = create_gui_event_sender_receiver();

    let sound_generator_factory = SoundGeneratorFactory::new();
    let control_channel = sound_generator_factory.event_sender.clone();
    let synth_state_receiver = sound_generator_factory.state_receiver.clone();

    let mut player = Player::new(sound_generator_factory, None);

    let sample_rate = player.start().unwrap();

    let mut granular_osc_sender = GranularOscMessageSender::new(
        "127.0.0.1:9664".to_owned(),
        "127.0.0.1:9665".to_owned(),
        &gui_recv,
    );
    let mut synth_controller =
        GranularController::new(control_channel, Some(gui_send), sample_rate);
    let _ = synth_controller
        .load_samples("/home/deman/projets/perso/rust/granular/mission24000.wav".to_owned());

    let wrapped_synth_controller = Arc::new(Mutex::new(synth_controller));
    let mut osc_receiver = OscReceiver::new(
        "127.0.0.1:9666".to_owned(),
        GranularOscMessageHandler::new(wrapped_synth_controller.clone()),
    );
    granular_osc_sender.start().unwrap();
    osc_receiver.start().unwrap();

    synth_to_ctrl_state_synchro(synth_state_receiver, wrapped_synth_controller.clone());

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
