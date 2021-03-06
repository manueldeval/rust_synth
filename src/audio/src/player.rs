use crossbeam::channel::{bounded, Sender};
use dsp::core::StereoBuffer;

use crate::{
    audioengine::AudioEngine,
    synthengine::{StereoGeneratorFactory, SynthEngine},
};

pub struct Player<T>
where
    T: StereoGeneratorFactory + 'static,
{
    synth_engine: SynthEngine<T>,
    audio_engine: AudioEngine,
}

impl<T> Player<T>
where
    T: StereoGeneratorFactory + 'static,
{
    pub fn new(factory: T, auxiliary_sender: Option<Sender<StereoBuffer>>) -> Self {
        let (sender, receiver) = bounded(1);

        let synth_engine = SynthEngine::new(sender, auxiliary_sender, factory);
        let audio_engine = AudioEngine::new(receiver);

        Player {
            synth_engine,
            audio_engine,
        }
    }

    pub fn start(&mut self) -> Result<f32, anyhow::Error> {
        let sample_rate = self.audio_engine.start()?;
        self.synth_engine.start(sample_rate);

        Ok(sample_rate)
    }
    pub fn stop(&mut self) {
        self.synth_engine.stop();
        self.audio_engine.stop();
    }
}
