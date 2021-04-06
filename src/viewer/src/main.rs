// use std::{
//     sync::{Arc, Mutex},
//     thread,
// };

// use audio::{player::Player, synthengine::StereoGeneratorFactory};
// use crossbeam::channel::unbounded;
// use dsp::core::OscReceiver;
// use granular::{
//     create_synth_event_sender_receiver, GranularSynth, SynthEventReceiver, SynthEventSender,
// };
// use granular::{GranularController, GranularOscMessageHandler};
// use nannou::ui::prelude::*;
// use nannou::{prelude::*, ui::widget::Id, Ui};

// //===================================
// // Synth factory
// #[derive(Clone)]
// struct SoundGeneratorFactory {
//     event_sender: SynthEventSender,
//     event_receiver: SynthEventReceiver,
// }
// impl SoundGeneratorFactory {
//     pub fn new() -> Self {
//         let (s, r) = create_synth_event_sender_receiver();
//         SoundGeneratorFactory {
//             event_sender: s,
//             event_receiver: r,
//         }
//     }
// }

// impl StereoGeneratorFactory for SoundGeneratorFactory {
//     type Gen = GranularSynth;
//     fn create(&self) -> Self::Gen {
//         let osc = GranularSynth::new(self.event_receiver.clone());
//         osc
//     }
// }

// fn main() -> anyhow::Result<()> {
//     nannou::app(model).update(update).simple_window(view).run();
//     Ok(())
// }

// struct Model {
//     _player: Player<SoundGeneratorFactory>,
//     _synth_controller: Arc<Mutex<GranularController>>,
//     _osc_receiver: OscReceiver<GranularOscMessageHandler>,
//     data: Arc<Mutex<Vec<f32>>>,
//     ui: Ui,
//     slider_id: Id,
//     slider_value: Arc<Mutex<f32>>,
// }

// fn model(app: &App) -> Model {
//     // Sound
//     let (sender, receiver) = unbounded();
//     let sound_generator_factory = SoundGeneratorFactory::new();
//     let control_channel = sound_generator_factory.event_sender.clone();
//     let mut player = Player::new(sound_generator_factory, Some(sender));
//     let sample_rate = player.start().unwrap();

//     let mut synth_controller = GranularController::new(control_channel, None, sample_rate);
//     let _ = synth_controller
//         .load_samples("/home/deman/projets/perso/rust/granular/mission24000.wav".to_owned());

//     let wrapped_synth_controller = Arc::new(Mutex::new(synth_controller));
//     let mut osc_receiver = OscReceiver::new(
//         "127.0.0.1:9666".to_owned(),
//         GranularOscMessageHandler::new(wrapped_synth_controller.clone()),
//     );
//     osc_receiver.start().unwrap();

//     //
//     let slider_value = Arc::new(Mutex::new(100.0_f32));
//     let data = Arc::new(Mutex::new(vec![0.0_f32, 0.0_f32]));
//     let other_data = data.clone();

//     let thread_slider_value = slider_value.clone();
//     thread::spawn(move || loop {
//         let size = { thread_slider_value.lock().unwrap().clone() } as usize;
//         let mut buffer: Vec<f32> = Vec::new();
//         for _ in 0..size {
//             let frames = receiver.recv().unwrap();
//             buffer.extend(frames[0].get())
//         }

//         let mut d = data.lock().unwrap();
//         d.clear();
//         buffer.iter().step_by(size / 20).for_each(|src| {
//             d.push(*src);
//         });
//     });

//     let mut ui = app.new_ui().build().unwrap();
//     let slider_id = ui.generate_widget_id();
//     Model {
//         _player: player,
//         data: other_data,
//         ui,
//         slider_id: slider_id,
//         slider_value: slider_value,
//         _synth_controller: wrapped_synth_controller.clone(),
//         _osc_receiver: osc_receiver,
//     }
// }

// fn update(_app: &App, model: &mut Model, _update: Update) {
//     let mut slider_value = model.slider_value.lock().unwrap();
//     let size = slider_value.clone() as f32;

//     for value in widget::Slider::new(size, 20.0, 500.0)
//         .w_h(400.0, 30.0)
//         .label_font_size(15)
//         .rgb(0.3, 0.3, 0.3)
//         .label_rgb(1.0, 1.0, 1.0)
//         .border(0.0)
//         .top_left_with_margin(20.0)
//         .label(&format!("Scale: {}", size as usize))
//         .set(model.slider_id, &mut model.ui.set_widgets())
//     {
//         *slider_value = value;
//     }
// }

// fn view(app: &App, model: &Model, frame: Frame) {
//     let d = model.data.lock().unwrap();
//     // Begin drawing
//     let draw = app.draw();

//     // Clear the background.
//     draw.background().color(BLACK);

//     let win = app.window_rect();

//     // Decide on a number of points and a weight.
//     let weight = 1.0;
//     let vertices = d.iter().enumerate().map(|(i, &amp)| {
//         let x = map_range(i, 0, d.len() - 1, win.left(), win.right());
//         let y = map_range(amp, -1.0, 1.0, win.bottom() * 0.75, win.top() * 0.75);
//         ((x, y), STEELBLUE)
//     });

//     draw.line()
//         .start(pt2(win.left(), 0.0))
//         .end(pt2(win.right(), 0.0))
//         .weight(1.0)
//         .color(WHITE);
//     draw.polyline()
//         .weight(weight)
//         .join_round()
//         .points_colored(vertices);

//     // Write the result of our drawing to the window's frame.
//     draw.to_frame(app, &frame).unwrap();
//     model.ui.draw_to_frame(app, &frame).unwrap();
// }
fn main(){
    
}