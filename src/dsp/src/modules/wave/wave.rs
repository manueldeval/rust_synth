pub trait Wave {
    fn get_at(&self, location: f32) -> f32;
    fn get_at_with_phase(&self, location: f32, phase_offset: f32) -> f32 {
        self.get_at(self.location_from_phase(location, phase_offset))
    }
    fn get_at_zero_one(&self, location: f32) -> f32 {
        1.0 + self.get_at(location) / 2.0
    }
    fn get_at_zero_one_with_phase(&self, location: f32, phase_offset: f32) -> f32 {
        self.get_at_zero_one(self.location_from_phase(location, phase_offset))
    }

    fn location_from_phase(&self, location: f32, phase_offset: f32) -> f32 {
        let location_plus_phase = phase_offset + location;
        if location_plus_phase > 1.0 {
            location_plus_phase - 1.0
        } else {
            location
        }
    }

    fn to_zero_one(value: f32) -> f32 {
        (1.0 + value) / 2.0
    }
}

const WAVE_SIN_SIZE: usize = 44100;
lazy_static! {
    static ref SIN: [f32; WAVE_SIN_SIZE] = {
        let mut result: [f32; WAVE_SIN_SIZE] = [0.0; WAVE_SIN_SIZE];
        for (x, y) in result.iter_mut().zip(0..WAVE_SIN_SIZE) {
            *x = (y as f32 * 2.0 * std::f32::consts::PI / WAVE_SIN_SIZE as f32).sin();
        }
        result
    };
}

pub struct SineWave {}

impl Wave for SineWave {
    fn get_at(&self, location: f32) -> f32 {
        match location {
            e if e < 0.0 => SIN[0],
            e if e >= 1.0 => SIN[WAVE_SIN_SIZE - 1],
            _ => {
                let idx_as_float = WAVE_SIN_SIZE as f32 * location;
                let idx1 = idx_as_float as usize;
                let idx2 = idx1 + 1;
                if idx2 < WAVE_SIN_SIZE {
                    let weight = idx_as_float.fract();
                    SIN[idx1] * (1.0 - weight) + SIN[idx2] * weight
                } else {
                    SIN[idx1]
                }
            }
        }
    }
}

pub struct SquareWave {}

impl Wave for SquareWave {
    fn get_at(&self, location: f32) -> f32 {
        match location {
            e if e < 0.5 => -1.0,
            _ => 1.0,
        }
    }
}

pub struct SawWave {}

impl Wave for SawWave {
    fn get_at(&self, location: f32) -> f32 {
        match location {
            e if e < 0.0 => -1.0,
            e if e > 1.0 => 1.0,
            _ => -1.0 + (location * 2.0),
        }
    }
}

pub struct TriangleWave {}

impl Wave for TriangleWave {
    fn get_at(&self, location: f32) -> f32 {
        match location {
            e if e < 0.0 => -1.0,
            e if e > 1.0 => 1.0,
            e if e < 0.5 => -1.0 + 4.0 * location,
            e if e > 0.5 => 3.0 - 4.0 * location,
            _ => -1.0,
        }
    }
}
