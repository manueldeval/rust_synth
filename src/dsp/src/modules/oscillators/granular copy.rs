use std::{cell::RefCell, rc::Rc};

use crate::core::{Buffer, Module, MonoGenerator, SharedBuffer, StereoGenerator};

#[derive(PartialEq)]
enum GrainState {
    SlopeUp,
    SlopeDown,
    Release,
    Inactive,
}

impl Into<usize> for GrainState {
    fn into(self) -> usize {
        match self {
            GrainState::Inactive => 4,
            GrainState::SlopeDown => 3,
            GrainState::SlopeUp => 2,
            GrainState::Release => 1,
        }
    }
}

struct Grain {
    state: GrainState,
    level: f32,
    location: usize,
    release_time: usize,
}

impl Grain {
    pub fn new() -> Self {
        Self {
            state: GrainState::Inactive,
            level: 0.0,
            location: 0,
            release_time: 0,
        }
    }

    pub fn restart(&mut self, location: usize) {
        self.location = location;
        self.release_time = 0;
        self.state = GrainState::SlopeUp;
        self.level = 0.0;
    }

    pub fn next(&mut self, slope_step: f32, duration: usize) -> Option<(usize, f32)> {
        self.location += 1;

        match &self.state {
            GrainState::Inactive => None,
            GrainState::Release => {
                self.release_time += 1;

                if self.release_time > duration {
                    self.state = GrainState::SlopeDown;
                };
                Some((self.location, 1.0))
            }
            GrainState::SlopeUp => {
                let new_level = self.level + slope_step;
                self.level = new_level.min(1.0);

                if self.level == 1.0 {
                    self.state = GrainState::Release;
                }
                Some((self.location, self.level))
            }
            GrainState::SlopeDown => {
                let new_level = self.level - slope_step;

                self.level = new_level.max(0.0);

                if self.level == 0.0 {
                    self.state = GrainState::Inactive;
                }
                Some((self.location, self.level))
            }
        }
    }
}

struct Grains {
    grains: [Grain; 2],
    slope_step: f32,
    duration: usize,
    grain_offset: usize,
    current_grain_offset: usize,
    _sample_length: usize,
    start: usize,
    end: usize,
    current_position: usize,
}

impl Grains {
    fn new(sample_length: usize) -> Self {
        Self {
            grains: [Grain::new(), Grain::new()],
            slope_step: 0.05,
            duration: 10,
            grain_offset: 0,
            current_grain_offset: 0,
            _sample_length: sample_length,
            start: 0,
            end: if sample_length > 0 {
                sample_length - 1
            } else {
                0
            },
            current_position: 0,
        }
    }
    pub fn get_available_grain_index(&self) -> Option<usize> {
        self.grains
            .iter()
            .position(|g| g.state == GrainState::Inactive)
    }

    pub fn must_respawn_new_grain(&self) -> bool {
        // We can respan an new grain if there is actually no
        // SlopeUp and Release state
        self.grains[0].state != GrainState::SlopeUp
            && self.grains[0].state != GrainState::Release
            && self.grains[1].state != GrainState::SlopeUp
            && self.grains[1].state != GrainState::Release
    }

    pub fn next(&mut self) -> Vec<(usize, f32)> {
        // If inactive, create one!
        if self.must_respawn_new_grain() {
            match self.get_available_grain_index() {
                Some(idx) => {
                    // Temporization beetween two grains
                    if self.current_grain_offset >= self.grain_offset {
                        //let pos = ((self.end - self.start) as f32 / 2.0) * (1.0+(2.0*std::f32::consts::PI * (self.current_position - self.start) as f32 /(self.end - self.start)as f32 ) .sin())/2.0 + self.start as f32;

                        let pos = self.current_position / 2;

                        self.grains[idx].restart(pos as usize);
                        //self.grains[idx].restart(self.current_position);

                        // Reset the cooldown beetween each grain.
                        self.current_grain_offset = 0;
                    } else {
                        self.current_grain_offset += 1;
                    }
                }
                None => {}
            }
        };

        // Update position (todo: non linear)
        if self.current_position >= self.end {
            self.current_position = self.start;
        }
        // if self.current_position <= self.start {
        //     self.current_position = self.end - 1;
        // }
        else {
            self.current_position += 1;
        }

        // Compute
        let mut result: Vec<(usize, f32)> = vec![];
        for g in self.grains.iter_mut() {
            let next_result = g.next(self.slope_step, self.duration);
            match next_result {
                Some(tuple) => result.push(tuple),
                None => {}
            };
        }

        result
    }
}

pub struct Granular {
    original_wav: Vec<f32>,
    original_sample_rate: f32,
    converted_wav: Vec<f32>,
    output: SharedBuffer,
    _index: usize,
    sample_rate: f32,

    // Granular
    grains: Grains,
}

impl Granular {
    pub fn new(sound_file: &str) -> Self {
        let mut reader = hound::WavReader::open(sound_file).unwrap();
        let spec = reader.spec();
        println!("Channels: {}", spec.channels);
        println!("Sample rate: {}", spec.sample_rate);
        println!("Bits_per_sample: {}", spec.bits_per_sample);
        let original_sample_rate = spec.sample_rate as f32;
        // Todo concert to target sample rate.
        let max_value = 2.0f32.powi(spec.bits_per_sample as i32);
        let data: Vec<f32> = reader
            .samples::<i32>()
            .map(|e| e.unwrap())
            .step_by(spec.channels as usize)
            .map(|v| (v as f32) / max_value)
            .collect();

        let default_sample_rate = 44_100_f32;
        let mut samples = Granular {
            _index: 0,
            original_wav: data,
            output: Rc::new(RefCell::new(Buffer::new())),
            sample_rate: default_sample_rate,
            original_sample_rate,
            converted_wav: vec![],

            // GRANULATOR!
            grains: Grains::new(0),
        };
        samples.set_sample_rate(default_sample_rate);
        samples
    }

    fn convert_sample_rate(&mut self) {
        self.converted_wav = vec![];
        if self.original_sample_rate == self.sample_rate {
            // Copy
            for v in &self.original_wav {
                self.converted_wav.push(*v);
            }
        } else {
            let ratio = self.sample_rate / self.original_sample_rate;

            if ratio < 1.0 {
                // Down sampling
                let new_size = (ratio * self.original_wav.len() as f32).round() as usize;
                for target_idx in 0..new_size {
                    let source_idx = (target_idx as f32 / ratio).round() as usize;
                    self.converted_wav.push(self.original_wav[source_idx]);
                }
            } else {
                // Upper sampling
                let new_size = (ratio * self.original_wav.len() as f32).round() as usize;
                for target_idx in 0..new_size {
                    let source_idx = (target_idx as f32 / ratio).trunc() as usize;
                    let coef = (target_idx as f32 / ratio).fract();
                    if source_idx + 1 >= self.original_wav.len() {
                        break;
                    }
                    self.converted_wav.push(
                        (1.0 - coef) * self.original_wav[source_idx]
                            + self.original_wav[source_idx + 1] * coef,
                    );
                }
            }
        }
        self.grains = Grains::new(self.converted_wav.len());
        self.grains.start = 0;
        self.grains.end = self.converted_wav.len();
        self.grains.current_position = 0;
    }
}

impl Module for Granular {
    fn process(&mut self) {
        let mut wrapped_buf = self.output.try_borrow_mut().unwrap();
        let buf = wrapped_buf.get_mut();

        for b in buf {
            *b = self
                .grains
                .next()
                .iter()
                .fold(0.0, |sum: f32, (pos, level)| {
                    if *pos >= self.converted_wav.len() {
                        0.0
                    } else {
                        sum + level * self.converted_wav[*pos]
                    }
                });
        }
        // for b in buf {
        //     *b = self.converted_wav[self.index];
        //     self.index += 1;

        //     self.index = if self.index < self.converted_wav.len() {
        //         self.index
        //     } else {
        //         0
        //     }
        // }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.convert_sample_rate();
    }
}

impl MonoGenerator for Granular {
    fn get_output(&self) -> SharedBuffer {
        self.output.clone()
    }
}

impl StereoGenerator for Granular {
    fn get_left_output(&self) -> SharedBuffer {
        self.output.clone()
    }

    fn get_right_output(&self) -> SharedBuffer {
        self.output.clone()
    }
}
