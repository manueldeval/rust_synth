use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::channel::{unbounded, Receiver, Sender};

use anyhow::anyhow;
use dsp::core::{stereo_buffer, Buffer, StereoBuffer, StereoFrame};
use std::thread;

pub struct StereoStream {
    audio_buffer_receiver: Receiver<StereoBuffer>,
    current_idx: usize,
    audio_buffer: StereoBuffer,
}

impl StereoStream {
    pub fn new(audio_buffer_receiver: Receiver<StereoBuffer>) -> Self {
        StereoStream {
            audio_buffer_receiver,
            current_idx: Buffer::size(),
            audio_buffer: stereo_buffer(),
        }
    }
}

impl Iterator for StereoStream {
    type Item = StereoFrame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= Buffer::size() {
            // End of buffer... load next chunk
            let audio_received = self.audio_buffer_receiver.recv().unwrap();

            // Copy left
            for (index, lval) in audio_received[0].get().iter().enumerate() {
                self.audio_buffer[0].get_mut()[index] = *lval;
            }

            // Copy right
            for (index, lval) in audio_received[1].get().iter().enumerate() {
                self.audio_buffer[1].get_mut()[index] = *lval;
            }

            self.current_idx = 0
        }
        let frame = Some([
            self.audio_buffer[0].get()[self.current_idx],
            self.audio_buffer[1].get()[self.current_idx],
        ]);
        self.current_idx += 1;
        frame
    }
}

enum AudioEngineCommand {
    Stop,
}

pub struct AudioEngine {
    command_sender: Sender<AudioEngineCommand>,
    command_receiver: Receiver<AudioEngineCommand>,
    audio_buffer_receiver: Receiver<StereoBuffer>,
}

impl AudioEngine {
    pub fn new(audio_buffer_receiver: Receiver<StereoBuffer>) -> AudioEngine {
        let (command_sender, command_receiver) = unbounded();

        AudioEngine {
            command_sender,
            command_receiver,
            audio_buffer_receiver,
        }
    }

    pub fn start(&mut self) -> Result<f32, anyhow::Error> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(anyhow!("Unable to get output device!"))?;
        let config = device.default_output_config()?;

        println!("Output device: {}", device.name()?);
        println!("Default output config: {:?}", config);
        let sample_rate = config.sample_rate().0 as f32;
        let sample_format = config.sample_format();

        let stream_config: cpal::StreamConfig = config.into();
        if stream_config.channels as usize != 2 {
            return Err(anyhow!("Only stereo devices are supported!"));
        };

        match sample_format {
            cpal::SampleFormat::F32 => self.run::<f32>(device, stream_config),
            cpal::SampleFormat::I16 => self.run::<i16>(device, stream_config),
            cpal::SampleFormat::U16 => self.run::<u16>(device, stream_config),
        };
        return Ok(sample_rate);
    }

    fn run<T>(&mut self, device: cpal::Device, config: cpal::StreamConfig)
    where
        T: cpal::Sample,
    {
        let command_sender = self.command_sender.clone();
        let err_fn = move |err| {
            let _ = command_sender.send(AudioEngineCommand::Stop);
            eprintln!("an error occurred on stream: {}", err);
        };

        let command_receiver = self.command_receiver.clone();
        let mut stream = StereoStream::new(self.audio_buffer_receiver.clone());
        thread::spawn(move || {
            let stream = device
                .build_output_stream(
                    &config,
                    move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                        for frame in data.chunks_mut(2) {
                            let next_frame = stream.next().unwrap();
                            frame[0] = cpal::Sample::from::<f32>(&next_frame[0]);
                            frame[1] = cpal::Sample::from::<f32>(&next_frame[1]);
                        }
                    },
                    err_fn,
                )
                .unwrap();
            let _stream = stream.play();
            // Wait for termination...
            let _ = command_receiver.recv();
        });
    }

    pub fn stop(&self) {
        let _ = self.command_sender.send(AudioEngineCommand::Stop);
    }
}
