use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::core::{Buffer, Module, MonoGenerator, SharedBuffer, StereoGenerator};

// Adder
pub struct Adder {
    adder_output: SharedBuffer,
    adder_inputs: Vec<SharedBuffer>,
}

impl Adder {
    pub fn new() -> Self {
        Adder {
            adder_output: Rc::new(RefCell::new(Buffer::new())),
            adder_inputs: Vec::new(),
        }
    }
    pub fn add_input(&mut self, input: SharedBuffer) {
        self.adder_inputs.push(input);
    }
    pub fn set_inputs(&mut self, inputs: &Vec<SharedBuffer>) {
        self.adder_inputs = inputs.iter().map(|i| i.clone()).collect();
    }
    pub fn get_output(&self) -> SharedBuffer {
        self.adder_output.clone()
    }
}

impl Module for Adder {
    fn process(&mut self) {
        let mut a = self.adder_output.try_borrow_mut().unwrap();
        let output = a.get_mut();
        let inputs: Vec<Ref<Buffer>> = self
            .adder_inputs
            .iter()
            .map(|wrapped_buffer| wrapped_buffer.try_borrow().unwrap())
            .collect();

        for i in 0..Buffer::size() {
            let mut value = 0.0;
            for input in &inputs {
                value += input.get()[i];
            }
            output[i] = value;
        }
    }
}

impl MonoGenerator for Adder {
    fn get_output(&self) -> SharedBuffer {
        Adder::get_output(&self)
    }
}

impl StereoGenerator for Adder {
    fn get_left_output(&self) -> SharedBuffer {
        Adder::get_output(&self)
    }

    fn get_right_output(&self) -> SharedBuffer {
        Adder::get_output(&self)
    }
}
