use std::{io, io::prelude::*};

extern crate anyhow;

extern crate audio;

use dsp::modules::{oscillators::BaseOscillator, wave::SineWave};
use audio::{player::{Player}, synthengine::{StereoGeneratorFactory}};



#[derive(Clone)]
struct SoundGeneratorFactory;
impl StereoGeneratorFactory for SoundGeneratorFactory {
    type Gen = BaseOscillator<SineWave>;
    fn create(&self) -> Self::Gen {
        BaseOscillator::new(SineWave {})
    }
}
impl SoundGeneratorFactory {
    pub fn new() -> Self { SoundGeneratorFactory {}}
}

fn main() -> anyhow::Result<()> {
    let mut player = Player::new(SoundGeneratorFactory::new(), None);
    player.start()?;
    pause();
    player.stop();
    Ok(())
}


fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}