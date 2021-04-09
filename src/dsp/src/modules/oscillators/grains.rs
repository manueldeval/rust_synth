
use crate::modules::wave::{CosWave, SineWave, Wave};

const COS_WAVE: CosWave = CosWave{};
const SIN_WAVE: SineWave = SineWave{};

pub fn get_pan(position: f32) -> (f32,f32) {
    let index = match position {
        p if p > 1.0 => { 0.0 },
        p if p < - 1.0 => { 0.25 },
        _ => { (1.0 + position)/8.0 }
    };
    (SIN_WAVE.get_at(index),COS_WAVE.get_at(index))
} 


#[derive(Debug,PartialEq)]
pub enum GrainState {
    Attack,
    Sustain,
    Release,
    Stopped,
}


pub struct Grain {
    position: f32,
    state: GrainState,
    l_coef: f32,
    r_coef: f32,
    attack_slope: f32,
    sustain_time: f32,
    release_slope: f32,
    value: f32,
}

pub struct GrainResult {
    pub position: f32,
    pub l_value: f32,
    pub r_value: f32,
}

impl Grain {
    pub fn new() -> Self {
        let position= 0.0;
        let l_coef = 0.0;
        let r_coef = 0.0;
        let state = GrainState::Stopped;
        let attack_slope = 0.0;
        let sustain_time = 0.0;
        let release_slope = 0.0;
        let value = 0.0;
        Grain {
            position,
            state,
            l_coef,
            r_coef,
            attack_slope,
            sustain_time,
            release_slope,
            value,
        }
    }

    #[inline]
    pub fn next(&mut self, step: f32, start: f32, end: f32) -> Option<GrainResult> {
        let opt_value = match self.state {
            GrainState::Attack => {
                self.value += self.attack_slope;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.state = GrainState::Sustain;
                };
                Some(self.value)
            },
            GrainState::Sustain => {
                self.sustain_time -= 1.0;
                if self.sustain_time <= 0.0 {
                    self.state = GrainState::Release;
                }
                Some(1.0)
            },
            GrainState::Release => {
                self.value += self.release_slope;
                if self.value < 0.0 {
                    self.value = 0.0;
                    self.state = GrainState::Stopped;
                };
                Some(self.value)
            }
            GrainState::Stopped => None,
        };
        
        opt_value.map(|value| {
            let new_position = self.position + step;
            // Todo change pos...
            self.position = match new_position {
                p if p >= end => start,
                p if p < start => end - 1.0,
                p => p
            };
            GrainResult {
                position: self.position, 
                l_value: value * self.l_coef,
                r_value: value * self.r_coef,
            }
        })
    }

    pub fn recycle(&mut self,position: f32, pan_spread: f32, location_spread: f32, attack_slope: f32, sustain_time: f32, release_slope: f32){
        let pan_angle = pan_spread * (2.0*fastrand::f32() - 1.0);
        let (l_coef,r_coef) = get_pan(pan_angle);

        let location_offset = location_spread * (2.0*fastrand::f32() - 1.0);

        self.position = position + location_offset;
        self.state = GrainState::Attack;
        self.r_coef = r_coef;
        self.l_coef = l_coef;
        self.value = 0.0;
        self.attack_slope = attack_slope;
        self.sustain_time = sustain_time;
        self.release_slope = release_slope;
    }
}

impl Default for Grain {
    fn default() -> Self {
        Self::new()
    }
}

pub const MAX_GRAINS: usize = 32;

pub struct Grains {
    // Orchestration
    time_beetween_grains: f32,
    current_time: f32,
    pub grains: Vec<Grain>,
}

impl Grains {
    pub fn new(time_beetween_grains: f32) -> Self {
        let current_time = 0.0;
        let mut grains = Vec::with_capacity(MAX_GRAINS);
        for _ in 0..MAX_GRAINS {
            grains.push(Grain::default());
        }
        Self {
            time_beetween_grains,
            current_time,
            grains,
        }
    }

    pub fn set_grain_density(&mut self, time_beetween_grains: f32){
        self.time_beetween_grains = time_beetween_grains;
    }

    // pan_spread 0 .. 1
    // location_spread: length of the sample/2
    pub fn grain_scheduler(&mut self, step: f32, scanner_location: f32, pan_spread: f32, location_spread: f32,attack_slope: f32, sustain_time: f32, release_slope: f32){
        self.current_time += step;
        if self.current_time > self.time_beetween_grains {
            // Create new grain if possible, if not we have to wait the next step...
            if self.create_grain(scanner_location, pan_spread, location_spread, attack_slope, sustain_time, release_slope){
                self.current_time = 0.0;
            } else {
                self.current_time = self.time_beetween_grains;
            }
        }
    }

    pub fn create_grain(&mut self, scanner_location: f32, pan_spread: f32, location_spread: f32, attack_slope: f32, sustain_time: f32, release_slope: f32) -> bool {
        let opt_grain = self.grains
            .iter_mut()
            .filter(|g| g.state == GrainState::Stopped)
            .nth(0);
        if let Some(g) = opt_grain {
            g.recycle(scanner_location, pan_spread, location_spread, attack_slope, sustain_time, release_slope);
            true
        } else {
            false
        }
    }
}

