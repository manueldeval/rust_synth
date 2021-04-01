use std::{cell::RefCell, f32, rc::Rc};

use crate::core::{Buffer, Module, MonoGenerator, SharedBuffer, StereoGenerator};

pub struct Samples {
    samples: Vec<f32>,
    output: SharedBuffer,
    index: f32,
    step: f32,
    sample_rate: f32,
    level: f32,
    start: usize, // Included
    end: usize,   // Excluded
}

impl Samples {
    pub fn new() -> Self {
        let index = 0.0;
        let step = 1.0;
        let sample_rate = 44_100_f32;
        let output = Rc::new(RefCell::new(Buffer::new()));
        let samples = vec![0.0, 0.0];
        let level = 1.0;
        let start = 0;
        let end = samples.len();
        Self {
            index,
            step,
            output,
            sample_rate,
            samples,
            level,
            start,
            end,
        }
    }

    pub fn load_samples(&mut self, samples: Vec<f32>) {
        self.samples = samples;
        self.index = 0.0;
        self.start = 0;
        self.end = self.samples.len();
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level;
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
        if self.index >= end as f32 {
            self.index = self.start as f32
        }
    }

    pub fn set_step(&mut self, step: f32) {
        self.step = step;
    }
}

impl Module for Samples {
    fn process(&mut self) {
        let mut wrapped_buf = self.output.try_borrow_mut().unwrap();
        let buf = wrapped_buf.get_mut();

        for b in buf {
            let i = self.index as usize;
            let w = self.index.fract();

            let v0 = self.samples[i];
            let v1 = self.samples[i + 1];

            *b = self.level * ((1.0 - w) * v0 + w * v1);

            self.index += self.step;
            // -1 because of the interpolation.
            if self.index >= self.end as f32 - 1.0 {
                self.index = self.start as f32
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl MonoGenerator for Samples {
    fn get_output(&self) -> SharedBuffer {
        self.output.clone()
    }
}

impl StereoGenerator for Samples {
    fn get_left_output(&self) -> SharedBuffer {
        self.output.clone()
    }

    fn get_right_output(&self) -> SharedBuffer {
        self.output.clone()
    }
}
