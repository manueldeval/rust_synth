use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::GuiEvent;
use crate::{GranularError, GuiEventSender, SynthEvent, SynthEventSender, SynthState};
use ring_channel::RingReceiver;

// =========================
// CONTROLLER
// =========================

pub struct GranularController {
    synth_event_sender: SynthEventSender,
    gui_event_sender: Option<GuiEventSender>,
    current_file: Option<String>,
    samples: Vec<f32>,
    sample_rate: f32,
    main_level: f32,
    start: f32,
    end: f32,
    semi_tones: f32,
    grain_semi_tones: f32,
    grain_attack_ratio: f32,
    grain_release_ratio: f32,
    grain_duration: f32,
}

impl GranularController {
    pub fn new(
        synth_event_sender: SynthEventSender,
        gui_event_sender: Option<GuiEventSender>,
        sample_rate: f32,
    ) -> Self {
        let current_file = None;
        let samples = vec![];
        let main_level = 1.0;
        let start = 0.0;
        let end = 0.0;
        let semi_tones = 0.0;
        let grain_semi_tones = 0.0;
        let grain_attack_ratio = 0.2;
        let grain_release_ratio = 0.2;
        let grain_duration = 1_000.0; 

        Self {
            synth_event_sender,
            gui_event_sender,
            current_file,
            samples,
            sample_rate,
            main_level,
            start,
            end,
            semi_tones,
            grain_semi_tones,
            grain_attack_ratio,
            grain_release_ratio,
            grain_duration
        }
    }
    pub fn set_level(&mut self, level: f32) {
        self.main_level = level;
        self.synth_event_sender.send(SynthEvent::MainLevel(level));
    }
    pub fn set_start(&mut self, start: f32) {
        if start < self.end {
            self.start = start;

            let start_at_sample = ((self.samples.len() as f32) * start) as usize;
            self.synth_event_sender
                .send(SynthEvent::Start(start_at_sample));
        }
    }
    pub fn set_end(&mut self, end: f32) {
        if end > self.start {
            self.end = end;

            let end_at_sample = ((self.samples.len() as f32) * end) as usize;
            self.synth_event_sender.send(SynthEvent::End(end_at_sample));
        }
    }
    pub fn set_tune(&mut self, semi_tones: f32) {
        self.semi_tones = semi_tones;
        let ratio = 2.0_f32.powf(semi_tones / 12.0);
        self.synth_event_sender.send(SynthEvent::Step(ratio));
    }

    pub fn set_pan_spread(&mut self,pan: f32){
        let pan = match pan {
            p if p <0.0 => 0.0,
            p if p> 1.0=> 1.0,
            p => p
        };
        self.synth_event_sender.send(SynthEvent::PanSpread(pan));
    }

    pub fn set_scan_spread(&mut self,spread: f32){
        let spread = match spread {
            p if p <0.0 => 0.0,
            p if p> 1.0=> 1.0,
            p => p
        };
        self.synth_event_sender.send(SynthEvent::ScanSpread(spread));

    }

    fn update_grain_env(&mut self){
        let attack_duration = self.grain_duration * self.grain_attack_ratio;
        let release_duration = self.grain_duration * self.grain_release_ratio;
        let sustain_duration = self.grain_duration - (attack_duration+release_duration);
        let sustain_duration = if sustain_duration > 0.0 { sustain_duration } else {0.0};

        self.synth_event_sender.send(SynthEvent::GrainEnvelop(1.0/attack_duration,sustain_duration,-1.0/release_duration));
    }

    pub fn set_grain_length(&mut self,length: f32){
        self.grain_duration = length;
        self.update_grain_env();
    }

    pub fn set_grain_attack_release_ratio(&mut self,attack_ratio: f32,release_ratio: f32){
        self.grain_attack_ratio = attack_ratio;
        self.grain_release_ratio = release_ratio;
        self.update_grain_env();
    }

    pub fn set_grain_tune(&mut self, semi_tones: f32) {
        self.grain_semi_tones = semi_tones;
        let ratio = 2.0_f32.powf(semi_tones / 12.0);
        println!("{}",ratio);
        self.synth_event_sender.send(SynthEvent::GrainStep(ratio));
    }

    pub fn set_grains_per_sec(&mut self, grains_per_sec: f32) {
        self.synth_event_sender.send(SynthEvent::GrainsPerSec(grains_per_sec));
    }

    pub fn update_synth_state(&mut self, state: SynthState) {
        if let Some(gui_sender) = &self.gui_event_sender {
            //println!("POSITION: {}/{}",self.samples.len() as f32,state.index);

            gui_sender.send(GuiEvent::Position(state.index / self.samples.len() as f32));
        }
    }

    pub fn load_samples(&mut self, file_name: String) -> Result<(), GranularError> {
        // Try to load samples
        self.current_file = Some(file_name.to_owned());
        let mut reader = hound::WavReader::open(&file_name)
            .map_err(|_e| GranularError::FailedToLoadSampleFile(file_name))?;
        let spec = reader.spec();
        println!("Channels: {}", spec.channels);
        println!("Sample rate: {}", spec.sample_rate);
        println!("Bits_per_sample: {}", spec.bits_per_sample);
        let original_sample_rate = spec.sample_rate as f32;

        // Get first channel and convert data -> -1<f32<1
        // Actually load only the fisrt channel.
        let max_value = 2.0_f32.powi(spec.bits_per_sample as i32);
        let original_data: Vec<f32> = reader
            .samples::<i32>()
            .map(|e| e.unwrap())
            .step_by(spec.channels as usize)
            .map(|v| (v as f32) / max_value)
            .collect();
        let ratio = self.sample_rate / original_sample_rate;
        let original_wav_len = original_data.len() as f32;

        // Resample
        self.samples.truncate(0);
        if ratio < 1.0 {
            // Down sampling
            let new_size = (ratio * original_wav_len).round() as usize;
            for target_idx in 0..new_size {
                let source_idx = (target_idx as f32 / ratio).round() as usize;
                self.samples.push(original_data[source_idx]);
            }
        } else {
            // Upper sampling
            let new_size = (ratio * original_wav_len).round() as usize;
            for target_idx in 0..new_size {
                let source_idx = (target_idx as f32 / ratio).trunc() as usize;
                let coef = (target_idx as f32 / ratio).fract();
                if source_idx + 1 >= original_wav_len as usize {
                    break;
                }
                self.samples.push(
                    (1.0 - coef) * original_data[source_idx] + original_data[source_idx + 1] * coef,
                );
            }
        }

        // Send data to Synth
        let vec_to_send_to_synth: Vec<f32> = self.samples.iter().copied().collect();
        self.synth_event_sender
            .send(SynthEvent::LoadSound(vec_to_send_to_synth));

        // Send data to gui
        if let Some(gui_sender) = &self.gui_event_sender {
            let window_size = self.samples.len() / 600;
            let vec_to_send_to_gui: Vec<f32> = if window_size <= 1 {
                vec![0.0]
            } else {
                let w = self.samples.chunks(window_size);
                let rms: Vec<f32> = w
                    .into_iter()
                    .map(|w| w.iter().map(|v| *v * *v).sum::<f32>().sqrt())
                    .collect();
                let mut mx: f32 = 0.0;
                for v in &rms {
                    if *v > mx {
                        mx = *v;
                    }
                }
                rms.iter().map(|v| v / mx).collect()
            };
            gui_sender.send(GuiEvent::SampleRms(vec_to_send_to_gui));
        }

        Ok(())
    }
}

pub fn synth_to_ctrl_state_synchro(
    synth_state_receiver: RingReceiver<SynthState>,
    wrapped_synth_controller: Arc<Mutex<GranularController>>,
) {
    // Syncho state!
    let mut synth_state_receiver = synth_state_receiver.clone();
    thread::spawn(move || loop {
        if let Ok(state) = synth_state_receiver.recv() {
            let mut ctrl = wrapped_synth_controller.lock().unwrap();
            ctrl.update_synth_state(state);
        }
    });
}
