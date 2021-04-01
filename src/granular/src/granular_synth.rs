use std::{cell::RefCell, rc::Rc};

use dsp::{
    core::{Module, SharedBuffer, StereoGenerator},
    modules::oscillators::Samples,
};

use crate::event::{Event, EventReceiver};

// =========================
// SYNTH
// =========================
pub struct GranularSynth {
    granular_osc: Rc<RefCell<Samples>>,

    output: Rc<RefCell<dyn StereoGenerator>>,
    all: Vec<Rc<RefCell<dyn Module>>>,
    event_receiver: EventReceiver,
}

impl GranularSynth {
    pub fn new(recv: EventReceiver) -> Self {
        let granular_osc = Rc::new(RefCell::new(Samples::new()));

        let all: Vec<Rc<RefCell<dyn Module>>> = vec![granular_osc.clone()];

        Self {
            output: granular_osc.clone(),
            granular_osc,
            all,
            event_receiver: recv,
        }
    }

    pub fn handle_event(&mut self) {
        if let Some(event) = self.event_receiver.receive() {
            let mut granular = self.granular_osc.try_borrow_mut().unwrap();

            match event {
                Event::LoadSound(samples) => {
                    granular.load_samples(samples);
                }
                Event::MainLevel(level) => {
                    granular.set_level(level);
                }
                Event::Start(start) => {
                    granular.set_start(start);
                }
                Event::End(end) => {
                    granular.set_end(end);
                }
                Event::Step(step) => {
                    granular.set_step(step);
                }
            }
        };
    }
}

impl Module for GranularSynth {
    fn process(&mut self) {
        self.handle_event();

        self.all
            .iter()
            .for_each(|m| m.try_borrow_mut().unwrap().process());
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
