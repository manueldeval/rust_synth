use std::{cell::RefCell, rc::Rc};

use dsp::{
    core::{Buffer, Module, SharedBuffer, StereoGenerator},
    modules::{
        ops::{Adder, Multiplier},
        oscillators::BaseOscillator,
        wave::{SineWave, TriangleWave},
    },
};
pub struct Granular {
    main_osc: Rc<RefCell<BaseOscillator<SineWave>>>,

    am_modulation: Rc<RefCell<BaseOscillator<TriangleWave>>>,
    am_adder: Rc<RefCell<Adder>>,
    am_multiply: Rc<RefCell<Multiplier>>,

    output: Rc<RefCell<dyn StereoGenerator>>,
    all: Vec<Rc<RefCell<dyn Module>>>,
}

impl Granular {
    pub fn new() -> Self {
        let mut main_osc = BaseOscillator::new(SineWave {});
        main_osc.set_level(0.5);

        let mut am_modulation = BaseOscillator::new(TriangleWave {});
        am_modulation.set_frequency(10.0);
        am_modulation.set_level(0.5);

        let mut am_adder = Adder::new();
        am_adder.add_input(am_modulation.get_left_output());
        am_adder.add_input(Rc::new(RefCell::new(Buffer::with_value(0.5))));

        let mut am_multiply = Multiplier::new();
        am_multiply.set_inputs(&vec![main_osc.get_left_output(), am_adder.get_output()]);

        let main_osc = Rc::new(RefCell::new(main_osc));
        let am_modulation = Rc::new(RefCell::new(am_modulation));
        let am_adder = Rc::new(RefCell::new(am_adder));
        let am_multiply = Rc::new(RefCell::new(am_multiply));

        let all: Vec<Rc<RefCell<dyn Module>>> = vec![
            main_osc.clone(),
            am_modulation.clone(),
            am_adder.clone(),
            am_multiply.clone(),
        ];
        Granular {
            output: am_multiply.clone(),
            main_osc,
            am_modulation,
            am_adder,
            am_multiply,
            all,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.main_osc.try_borrow_mut().unwrap().set_frequency(freq);
    }
}

impl Module for Granular {
    fn process(&mut self) {
        self.all
            .iter()
            .for_each(|m| m.try_borrow_mut().unwrap().process());
    }
}

impl StereoGenerator for Granular {
    fn get_left_output(&self) -> SharedBuffer {
        self.output.try_borrow().unwrap().get_left_output()
    }

    fn get_right_output(&self) -> SharedBuffer {
        self.output.try_borrow().unwrap().get_right_output()
    }
}
