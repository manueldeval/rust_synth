use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use dsp::core::{Module, StereoBuffer, StereoGenerator};
use std::thread;

enum SynthCommand {
    Stop,
}

pub trait StereoGeneratorFactory: Send + Clone {
    type Gen: StereoGenerator;

    fn create(&self) -> Self::Gen;
}

pub struct SynthEngine<T>
where
    T: StereoGeneratorFactory + 'static,
{
    audio_buffer_sender: Sender<StereoBuffer>,
    auxiliary_audio_buffer_sender: Option<Sender<StereoBuffer>>,
    command_sender: Sender<SynthCommand>,
    command_receiver: Receiver<SynthCommand>,
    factory: T,
}

impl<T> SynthEngine<T>
where
    T: StereoGeneratorFactory + 'static,
{
    pub fn new(
        audio_buffer_sender: Sender<StereoBuffer>,
        auxiliary_audio_buffer_sender: Option<Sender<StereoBuffer>>,
        factory: T,
    ) -> Self {
        let (command_sender, command_receiver) = unbounded();

        SynthEngine {
            audio_buffer_sender,
            auxiliary_audio_buffer_sender,
            command_sender,
            command_receiver,
            factory,
        }
    }

    pub fn start(&mut self) {
        let audio_buffer_sender = self.audio_buffer_sender.clone();
        let command_receiver = self.command_receiver.clone();

        let factory = self.factory.clone();
        let aux_opt = self.auxiliary_audio_buffer_sender.clone();

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

                        if let Some(aux) = aux_opt.as_ref() {
                            let _ = aux.send([l.clone_buffer(), r.clone_buffer()]);
                        };
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
