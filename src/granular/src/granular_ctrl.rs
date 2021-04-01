use crate::{
    errors::GranularError,
    event::{Event, EventSender},
};

// =========================
// CONTROLLER
// =========================

pub struct GranularController {
    event_sender: EventSender,
    current_file: Option<String>,
    samples: Vec<f32>,
    sample_rate: f32,
    main_level: f32,
    start: f32,
    end: f32,
    semi_tones: f32,
}

impl GranularController {
    pub fn new(event_sender: EventSender, sample_rate: f32) -> Self {
        let current_file = None;
        let samples = vec![];
        let main_level = 1.0;
        let start = 0.0;
        let end = 0.0;
        let semi_tones = 0.0;
        Self {
            event_sender,
            current_file,
            samples,
            sample_rate,
            main_level,
            start,
            end,
            semi_tones,
        }
    }
    pub fn set_level(&mut self, level: f32) {
        self.main_level = level;
        self.event_sender.send(Event::MainLevel(level));
    }
    pub fn set_start(&mut self, start: f32) {
        if start < self.end {
            self.start = start;

            let start_at_sample = ((self.samples.len() as f32) * start) as usize;
            self.event_sender.send(Event::Start(start_at_sample));
        }
    }
    pub fn set_end(&mut self, end: f32) {
        if end > self.start {
            self.end = end;

            let end_at_sample = ((self.samples.len() as f32) * end) as usize;
            self.event_sender.send(Event::End(end_at_sample));
        }
    }
    pub fn set_tune(&mut self, semi_tones: f32) {
        self.semi_tones = semi_tones;
        let ratio = 2.0_f32.powf(semi_tones / 12.0);
        self.event_sender.send(Event::Step(ratio));
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
        let vec_to_send: Vec<f32> = self.samples.iter().copied().collect();
        self.event_sender.send(Event::LoadSound(vec_to_send));

        Ok(())
    }
}
