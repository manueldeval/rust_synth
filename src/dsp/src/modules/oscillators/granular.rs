use std::{cell::RefCell, f32, rc::Rc};

use crate::core::{Buffer, Module, MonoGenerator, SharedBuffer, StereoGenerator};

use super::{GrainResult, Grains, MAX_GRAINS};

pub struct Granulator {
    samples: Vec<f32>,
    output_left: SharedBuffer,
    output_right: SharedBuffer,
    index: f32,
    step: f32,
    sample_rate: f32,
    level: f32,
    start: usize, // Included
    end: usize, 
    grains: Grains,  // Excluded
    pan_spread: f32,
    scan_spread: f32,
    grain_step: f32,
    grain_attack_slope: f32,
    grain_sustain_duration: f32,
    grain_release_slope: f32,
    vec_result: Vec<GrainResult>,
}

impl Granulator {
    pub fn new() -> Self {
        let index = 0.0;
        let step = 1.0;
        let sample_rate = 44_100_f32;
        let output_left = Rc::new(RefCell::new(Buffer::new()));
        let output_right = Rc::new(RefCell::new(Buffer::new()));

        let samples = vec![0.0, 0.0, 0.0, 0.0, 0.0];
        let level = 1.0;
        let start = 0;
        let end = samples.len();

        let grains = Grains::new(200.0);
        let pan_spread = 0.0;
        let scan_spread = 0.0;
        let grain_step = 1.0;

        let grain_attack_slope = 0.1;
        let grain_sustain_duration = 1_000.0;
        let grain_release_slope= -0.1;

        let vec_result  = Vec::with_capacity(MAX_GRAINS);
        Self {
            index,
            step,
            output_left,
            output_right,
            sample_rate,
            samples,
            level,
            start,
            end,
            grains,
            pan_spread,
            scan_spread,
            grain_step,
            grain_attack_slope,
            grain_sustain_duration,
            grain_release_slope,
            vec_result,
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
    pub fn set_grain_step(&mut self,step: f32){
        self.grain_step = step;
    }
    pub fn set_pan_spread(&mut self, pan: f32){
        self.pan_spread = pan;
    }

    pub fn set_grains_per_sec(&mut self, grains_per_sec: f32) {
        self.grains.set_grain_density(self.sample_rate / grains_per_sec);
    }

    pub fn set_scan_spread(&mut self,scan_spread: f32){
        self.scan_spread = scan_spread;
    }

    pub fn set_grain_attack_slope(&mut self,v: f32){
        self.grain_attack_slope = v;
    }
    pub fn set_grain_sustain_duration(&mut self,v: f32){
        self.grain_sustain_duration = v;
    }
    pub fn set_grain_release_slope(&mut self,v: f32){
        self.grain_release_slope = v;
    }

    fn next_index(&self) -> f32 {
        let new_index = self.index + self.step;
        // -1 because of the interpolation.
        if new_index >= self.end as f32 - 1.0 {
            self.start as f32
        } else {
            new_index
        }
    }

    pub fn current_index(&self) -> f32 {
        self.index
    }

    #[inline] 
    fn get_value_at(&self,position: usize) -> f32 {
        self.samples[position]
    }

    #[inline] 
    fn get_interpolated_value_at(&self,position: f32) -> f32 {
        let idx = position as usize;
        let w = position.fract();
        let v0 = self.samples[idx];
        let v1 = self.samples[idx + 1]; 
        (1.0 - w) * v0 + w * v1
    }
}

impl Module for Granulator {
    fn process(&mut self) {
        let mut wrapped_buf_left = self.output_left.try_borrow_mut().unwrap();
        let mut wrapped_buf_right = self.output_right.try_borrow_mut().unwrap();

        let buf_left = wrapped_buf_left.get_mut();
        let buf_right = wrapped_buf_right.get_mut();
        let grain_step = self.grain_step;
        let start = 0.0;
        let end = self.samples.len() as f32 - 1.0;
        let attack_slope = self.grain_attack_slope;
        let sustain_duration = self.grain_sustain_duration;
        let release_slope = self.grain_release_slope;

        for (b_l,b_r) in buf_left.iter_mut().zip(buf_right) {
            self.grains.grain_scheduler(grain_step,
                self.index, 
                self.pan_spread, 
                self.scan_spread * (self.samples.len() as f32/2.0), 
                attack_slope, 
                sustain_duration,
                release_slope);

            let mut l_value = 0.0;
            let mut r_value = 0.0;
            let mut left = 0.0;
            let mut right = 0.0;            
            {
                let result = &mut self.vec_result;
                result.clear();
                for g in self.grains.grains.iter_mut() {
                    let optional_result = g.next(grain_step,start,end);
                    if let Some(res) = optional_result {
                        result.push(res);
                    }
                }

                // self.grains.grains
                //     .iter_mut()
                //     .map(|g| g.next(grain_step,start,end))
                //     .filter_map(|g| g)
                //     .for_each(|src|{
                //         result.push(src);
                //     });
            }
            for r in self.vec_result.iter() {
                l_value += r.l_value;
                r_value += r.r_value;
                let sample_value = self.get_interpolated_value_at(r.position);
                left += r.l_value * sample_value;
                right += r.r_value * sample_value;
            }

            if l_value > 1.0 {
                left = left/l_value;
            } 
            if r_value > 1.0 {
                right = right/r_value;
            } 
            *b_l  = self.level * left;
            *b_r = self.level * right;
            self.index = self.next_index();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl MonoGenerator for Granulator {
    fn get_output(&self) -> SharedBuffer {
        self.output_left.clone()
    }
}

impl StereoGenerator for Granulator {
    fn get_left_output(&self) -> SharedBuffer {
        self.output_left.clone()
    }

    fn get_right_output(&self) -> SharedBuffer {
        self.output_right.clone()
    }
}
