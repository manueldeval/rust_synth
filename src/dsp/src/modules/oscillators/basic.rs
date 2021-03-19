use crate::core::{Module, SharedBuffer, StereoGenerator};

use crate::modules::wave::SawWave;
use crate::modules::wave::SineWave;
use crate::modules::wave::SquareWave;
use crate::modules::wave::TriangleWave;

use crate::modules::wave::Wave;

pub struct BaseOscillator<W: Wave> {
    sample_rate: f32,
    frequency: f32,
    step: f32,
    phase: f32,
    level: f32,
    wave: W,
    output: SharedBuffer,
}

impl<W: Wave> BaseOscillator<W> {
    pub fn new(wave: W) -> BaseOscillator<W> {
        let mut osc = BaseOscillator {
            sample_rate: 44100.0,
            frequency: 220.0,
            step: 0.0,
            phase: 0.0,
            level: 0.25,
            wave: wave,
            output: SharedBuffer::default(),
        };
        osc.set_frequency(220.0);
        osc
    }
    pub fn set_frequency(&mut self, frequency: f32) {
        self.step = frequency / self.sample_rate;
        self.frequency = frequency;
    }
    pub fn set_level(&mut self, level: f32) {
        self.level = level;
    }
    fn normalize_phase(&self, phase: f32) -> f32 {
        if phase > 1.0 {
            phase - 1.0
        } else {
            phase
        }
    }
}

impl<'a, W: Wave> Module for BaseOscillator<W> {
    fn set_sample_rate(&mut self, frequency: f32) {
        self.sample_rate = frequency;
    }

    fn process(&mut self) {
        let mut output = self.output.try_borrow_mut().unwrap();

        for l in output.get_mut() {
            let value = self.level * self.wave.get_at(self.phase);
            self.phase += self.step;
            self.phase = self.normalize_phase(self.phase);
            *l = value;
        }
    }
}

impl<'a, W: Wave> StereoGenerator for BaseOscillator<W> {
    fn get_left_output(&self) -> SharedBuffer {
        self.output.clone()
    }

    fn get_right_output(&self) -> SharedBuffer {
        self.output.clone()
    }
}

pub type SinOscillator = BaseOscillator<SineWave>;
pub type SquareOscillator = BaseOscillator<SquareWave>;
pub type SawOscillator = BaseOscillator<SawWave>;
pub type TriangleOscillator = BaseOscillator<TriangleWave>;
