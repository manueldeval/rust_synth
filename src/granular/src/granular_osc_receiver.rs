use std::sync::{Arc, Mutex};

use dsp::core::OscMessageHandler;
use rosc::{OscMessage, OscType};

use crate::GranularController;

// Osc receiver
pub struct GranularOscMessageHandler {
    synth_controller: Arc<Mutex<GranularController>>,
}

impl GranularOscMessageHandler {
    pub fn new(synth_controller: Arc<Mutex<GranularController>>) -> Self {
        Self { synth_controller }
    }
}

impl Clone for GranularOscMessageHandler {
    fn clone(&self) -> Self {
        Self {
            synth_controller: self.synth_controller.clone(),
        }
    }
}

impl OscMessageHandler for GranularOscMessageHandler {
    fn handle_message(&mut self, message: OscMessage) {
        let mut ctrl = self.synth_controller.lock().unwrap();
        match (&message.addr[..], message.args.as_slice()) {
            ("/sample", [OscType::String(e)]) => {
                println!("Change sample: {}", e);
                ctrl.load_samples(e.clone()).unwrap();
            }
            ("/level", [OscType::Float(e)]) => {
                println!("Change Level: {}", e);
                ctrl.set_level(*e);
            }
            ("/sample_bounds", [OscType::Float(start), OscType::Float(end)]) => {
                println!("Change Sample bounds: {} / {}", start, end);
                ctrl.set_start(*start);
                ctrl.set_end(*end);
            }
            ("/tune", [OscType::Float(semi_tones)]) => {
                println!("Change tune: {}", semi_tones);
                ctrl.set_tune(*semi_tones);
            }
            _ => {
                println!(
                    "No match for OSC address: {}, OSC arguments: {:?}",
                    message.addr, message.args
                );
            }
        };
    }
}
