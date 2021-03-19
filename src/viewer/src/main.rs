use std::{sync::{Arc, Mutex}, thread};

use audio::{player::Player, synthengine::{StereoGeneratorFactory}};
use crossbeam::channel::unbounded;
use dsp::{modules::{oscillators::BaseOscillator, wave::{SineWave}}};
use nannou::prelude::*;


#[derive(Clone)]
struct SoundGeneratorFactory;
impl StereoGeneratorFactory for SoundGeneratorFactory {
    type Gen = BaseOscillator<SineWave>;
    fn create(&self) -> Self::Gen {
        let mut osc = BaseOscillator::new(SineWave {});
        osc.set_frequency(110.0);
        osc
    }
}
impl SoundGeneratorFactory {
    pub fn new() -> Self { SoundGeneratorFactory {}}
}


fn main() -> anyhow::Result<()> {    
    nannou::app(model).simple_window(view).run();
    Ok(())
}

struct Model {
    _player: Player<SoundGeneratorFactory>,
    data: Arc<Mutex<Vec<f32>>>,
}

fn model(_app: &App) -> Model {
    let (sender, receiver) = unbounded();
    let mut player = Player::new(SoundGeneratorFactory::new(), Some(sender));
    let _ = player.start();
    
    let data = Arc::new(Mutex::new(vec!(0.0_f32,0.0_f32,)));
    let other_data = data.clone();
    thread::spawn(move||{
        loop {
            let mut buffer : Vec<f32> = Vec::new();
            for _ in 0..100 {
                let frames= receiver.recv().unwrap();
                buffer.extend(frames[0].get())
            }
            
            
            let mut d = data.lock().unwrap();
            d.clear();
            buffer.iter().step_by(5).for_each(|src|{
                d.push(*src);
            });
        }   
    });
    Model {
        _player: player,
        data: other_data,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let d = model.data.lock().unwrap();
    // Begin drawing
    let draw = app.draw();

    // Clear the background.
    draw.background().color(BLACK);

    let win = app.window_rect();

    // Decide on a number of points and a weight.
    let weight = 1.0;
    let vertices = d.iter().enumerate().map(|(i, &amp)| {
        let x = map_range(i, 0, d.len() - 1, win.left(), win.right());
        let y = map_range(amp, -1.0, 1.0, win.bottom() * 0.75, win.top() * 0.75);
        ((x, y), STEELBLUE)
    });

    draw.line()
        .start(pt2(win.left(), 0.0))
        .end(pt2(win.right(), 0.0))
        .weight(1.0)
        .color(WHITE);
    draw.polyline()
        .weight(weight)
        .join_round()
        .points_colored(vertices);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
