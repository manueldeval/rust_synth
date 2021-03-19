use std::{cell::RefCell, rc::Rc};

pub const N_SAMPLES_PER_CHUNK: usize = 32;

pub struct Buffer {
    buf: [f32; N_SAMPLES_PER_CHUNK],
}

impl Buffer {
    pub fn new() -> Self {
        Buffer::default()
    }

    pub fn with_value(value: f32) -> Buffer {
        Buffer {
            buf: [value; N_SAMPLES_PER_CHUNK],
        }
    }

    pub fn set_zero(&mut self) {
        *self = Default::default();
    }

    pub fn get(&self) -> &[f32; N_SAMPLES_PER_CHUNK] {
        &self.buf
    }

    pub fn get_mut(&mut self) -> &mut [f32; N_SAMPLES_PER_CHUNK] {
        &mut self.buf
    }

    pub fn size() -> usize {
        return N_SAMPLES_PER_CHUNK;
    }

    pub fn clone_buffer(&self) -> Buffer {
        let mut out = Buffer::default();
        for (self_elem, out_elem) in self.buf.iter().zip(out.get_mut().iter_mut()) {
            *out_elem = *self_elem;
        }
        out
    }
}

impl Default for Buffer {
    fn default() -> Buffer {
        Buffer {
            buf: [0.0; N_SAMPLES_PER_CHUNK],
        }
    }
}

pub type StereoBuffer = [Buffer; 2];
pub fn stereo_buffer() -> StereoBuffer {
    [Buffer::new(), Buffer::new()]
}

pub type StereoFrame = [f32; 2];

pub type SharedBuffer = Rc<RefCell<Buffer>>;
