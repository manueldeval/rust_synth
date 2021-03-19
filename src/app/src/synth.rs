use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use dsp::core::{StereoBuffer, StereoGenerator};
use std::marker::PhantomData;
use std::thread;

enum SynthCommand {
    Stop,
}

pub trait StereoGeneratorFactory<G>: Send + Clone
where
    G: StereoGenerator,
{
    fn create(&self) -> G;
}

pub struct Synth<T, G: StereoGenerator>
where
    T: StereoGeneratorFactory<G> + 'static,
{
    audio_buffer_sender: Sender<StereoBuffer>,
    command_sender: Sender<SynthCommand>,
    command_receiver: Receiver<SynthCommand>,
    factory: T,
    _marker: PhantomData<G>,
}

impl<T, G> Synth<T, G>
where
    T: StereoGeneratorFactory<G>,
    G: StereoGenerator + 'static,
{
    pub fn new(audio_buffer_sender: Sender<StereoBuffer>, factory: T) -> Self {
        let (command_sender, command_receiver) = unbounded();

        Synth {
            audio_buffer_sender,
            command_sender,
            command_receiver,
            factory,
            _marker: PhantomData,
        }
    }

    pub fn start(&mut self) {
        let audio_buffer_sender = self.audio_buffer_sender.clone();
        let command_receiver = self.command_receiver.clone();
        //let mut osc = BaseOscillator::new(SineWave {});
        let factory = self.factory.clone();
        thread::spawn(move || {
            // Create synth
            let mut stereo_generator = factory.create();
            // Play!
            loop {
                match command_receiver.try_recv() {
                    Err(TryRecvError::Empty) => {
                        stereo_generator.process();
                        let l = stereo_generator
                            .get_left_output()
                            .try_borrow()
                            .unwrap()
                            .clone_buffer();
                        let r = stereo_generator
                            .get_right_output()
                            .try_borrow()
                            .unwrap()
                            .clone_buffer();
                        let _ = audio_buffer_sender.send([l, r]);
                    }
                    _ => {
                        println!("Synth stopped.");
                        break;
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        let _ = self.command_sender.send(SynthCommand::Stop);
    }
}
