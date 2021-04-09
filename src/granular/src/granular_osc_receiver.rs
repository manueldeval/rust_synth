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
            },
            ("/pan_spread", [OscType::Float(spread)]) => {
                println!("Change pan spread: {}", spread);
                ctrl.set_pan_spread(*spread);
            },
            ("/scan_spread", [OscType::Float(spread)]) => {
                println!("Change scan spread: {}", spread);
                ctrl.set_scan_spread(*spread);
            }
            ("/grain_tune", [OscType::Float(tune)]) => {
                println!("Change grain tune: {}", tune);
                ctrl.set_grain_tune(*tune);
            }
            ("/grains_per_sec", [OscType::Float(grains_per_sec)]) => {
                println!("Change grains per sec: {}", grains_per_sec);
                ctrl.set_grains_per_sec(*grains_per_sec);
            }
            ("/grain_length", [OscType::Float(grain_length)]) => {
                println!("Change grain_length: {}", grain_length);
                ctrl.set_grain_length(*grain_length);
            }
            ("/grain_env", [OscType::Float(attack),OscType::Float(release)]) => {
                println!("Change grain_env: {} / {}", attack, release);
                ctrl.set_grain_attack_release_ratio(*attack,*release);
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
