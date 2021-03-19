use super::SharedBuffer;

pub trait Module {
    #[allow(unused)]
    fn set_sample_rate(&mut self, frequency: f32) {}

    #[allow(unused)]
    fn process(&mut self);
}

pub trait OuputAudioPin<M: Module> {
    fn index(&self) -> usize;
}

pub trait InputAudioPin<M: Module> {
    fn index(&self) -> usize;
}

pub trait StereoGenerator: Module {
    fn get_left_output(&self) -> SharedBuffer;
    fn get_right_output(&self) -> SharedBuffer;
}

pub trait MonoGenerator: Module {
    fn get_output(&self) -> SharedBuffer;
}

impl StereoGenerator for dyn MonoGenerator {
    fn get_left_output(&self) -> SharedBuffer {
        return self.get_output();
    }
    fn get_right_output(&self) -> SharedBuffer {
        return self.get_output();
    }
}
