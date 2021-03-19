use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::core::{Buffer, Module, MonoGenerator, SharedBuffer, StereoGenerator};

pub struct Multiplier {
    multiplier_output: SharedBuffer,
    multiplier_inputs: Vec<SharedBuffer>,
}

impl Multiplier {
    pub fn new() -> Self {
        Multiplier {
            multiplier_output: Rc::new(RefCell::new(Buffer::new())),
            multiplier_inputs: Vec::new(),
        }
    }
    pub fn add_input(&mut self, input: SharedBuffer) {
        self.multiplier_inputs.push(input);
    }
    pub fn set_inputs(&mut self, inputs: &Vec<SharedBuffer>) {
        self.multiplier_inputs = inputs.iter().map(|i| i.clone()).collect();
    }
    pub fn get_output(&self) -> SharedBuffer {
        self.multiplier_output.clone()
    }
}

impl Module for Multiplier {
    fn process(&mut self) {
        let mut a = self.multiplier_output.try_borrow_mut().unwrap();
        let output = a.get_mut();
        let inputs: Vec<Ref<Buffer>> = self
            .multiplier_inputs
            .iter()
            .map(|wrapped_buffer| wrapped_buffer.try_borrow().unwrap())
            .collect();

        for i in 0..Buffer::size() {
            output[i] = 1.0;
            for input in &inputs {
                output[i] *= input.get()[i];
            }
        }
    }
}

impl MonoGenerator for Multiplier {
    fn get_output(&self) -> SharedBuffer {
        Multiplier::get_output(&self)
    }
}

impl StereoGenerator for Multiplier {
    fn get_left_output(&self) -> SharedBuffer {
        Multiplier::get_output(&self)
    }

    fn get_right_output(&self) -> SharedBuffer {
        Multiplier::get_output(&self)
    }
}
