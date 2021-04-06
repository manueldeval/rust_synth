use std::{cell::RefCell, rc::Rc};
use dsp::{
    core::{Module, SharedBuffer, StereoGenerator},
    modules::oscillators::Samples,
};
use ring_channel::*;

use crate::event::{SynthEvent, SynthEventReceiver};


pub struct SynthState {
    pub index: f32
}

const STATE_COUNT: u16 = 50;
// =========================
// SYNTH
// =========================
pub struct GranularSynth {
    granular_osc: Rc<RefCell<Samples>>,

    output: Rc<RefCell<dyn StereoGenerator>>,
    all: Vec<Rc<RefCell<dyn Module>>>,
    event_receiver: SynthEventReceiver,
    state_sender: RingSender<SynthState>,
    state_count: u16,
}

impl GranularSynth {
    pub fn new(recv: SynthEventReceiver, state_sender: RingSender<SynthState>) -> Self {

        let granular_osc = Rc::new(RefCell::new(Samples::new()));
        let all: Vec<Rc<RefCell<dyn Module>>> = vec![granular_osc.clone()];

        Self {
            output: granular_osc.clone(),
            granular_osc,
            all,
            event_receiver: recv,
            state_sender,
            state_count: STATE_COUNT,
        }
    }

    pub fn handle_event(&mut self) {
        if let Some(event) = self.event_receiver.receive() {
            let mut granular = self.granular_osc.try_borrow_mut().unwrap();

            match event {
                SynthEvent::LoadSound(samples) => {
                    granular.load_samples(samples);
                }
                SynthEvent::MainLevel(level) => {
                    granular.set_level(level);
                }
                SynthEvent::Start(start) => {
                    granular.set_start(start);
                }
                SynthEvent::End(end) => {
                    granular.set_end(end);
                }
                SynthEvent::Step(step) => {
                    granular.set_step(step);
                }
            }
        };
    }

    pub fn send_state(&mut self){
        self.state_count -= 1;
        if self.state_count == 0 {
            self.state_count = STATE_COUNT;
            let granular = self.granular_osc.try_borrow_mut().unwrap();
            let _ = self.state_sender.send(SynthState {
                index: granular.current_index()
            });
        }
    }
}

impl Module for GranularSynth {
    fn process(&mut self) {
        // Handle events
        self.handle_event();

        // Process all
        self.all
            .iter()
            .for_each(|m| m.try_borrow_mut().unwrap().process());

        self.send_state();
    }

    fn set_sample_rate(&mut self, frequency: f32) {
        self.all
            .iter()
            .for_each(|m| m.try_borrow_mut().unwrap().set_sample_rate(frequency));
    }
}

impl StereoGenerator for GranularSynth {
    fn get_left_output(&self) -> SharedBuffer {
        self.output.try_borrow().unwrap().get_left_output()
    }

    fn get_right_output(&self) -> SharedBuffer {
        self.output.try_borrow().unwrap().get_right_output()
    }
}
