#![no_std]
// my own custom board abstractions go here

use core::cell::RefCell;
use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use critical_section_lock_mut::LockMut;
use microbit::hal::gpio::{self, p0::Parts, Output, PushPull};
use microbit::hal::prelude::*; // U32Ext
use microbit::hal::{delay::Delay, prelude, prelude::OutputPin, pwm, time::Hertz};
use microbit::pac::{self, interrupt};
use microbit::{board, Board};
use micromath::F32Ext;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// pub static BOARD: LockMut<board::Board> = LockMut::new();

static SPEAKER2: Mutex<RefCell<Option<pwm::Pwm<pac::PWM0>>>> = Mutex::new(RefCell::new(None));
pub static SPEAKER: LockMut<BoardState> = LockMut::new();

fn sine(samples: &mut [u16], q: u16, n: usize) {
    use core::f32::consts::PI;
    let step = 2.0 * PI * n as f32 / samples.len() as f32;
    for (i, s) in samples.iter_mut().enumerate() {
        // Get the next value.
        let v = libm::sinf(i as f32 * step);
        // Normalize to the range 0.0..=q-1.
        let v = (q - 1) as f32 * (v + 1.0) / 2.0;
        // Save a value in the range 0..=q-1.
        *s = libm::floorf(v + 0.5) as u16;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BoardState {
    Falling,
    NotFalling,
}

impl BoardState {
    pub fn new() -> Self {
        Self::NotFalling
    }

    pub fn next(self) -> BoardState {
        match self {
            // transition from falling to not falling
            // turn speaker off and change to default display
            Self::Falling => {
                // don't interrupt while changing state!
                cortex_m::interrupt::free(|_| {
                    self.speaker_off();
                    self.default_display();
                });
                Self::NotFalling
            }
            // transition from not falling to falling
            // turn speaker on and change to falling display
            Self::NotFalling => {
                // don't interrupt while changing state!
                cortex_m::interrupt::free(|_| {
                    self.speaker_on();
                    self.falling_display();
                });
                Self::Falling
            }
        }
    }

    pub fn speaker_on(&self) {
        rprintln!("turning speaker on!!!!!!!!!!!\n");
        unsafe {
            let p = pac::Peripherals::steal();
            let my_pins = Parts::new(p.P0);
            let speaker_pin = my_pins
                .p0_00
                .degrade()
                .into_push_pull_output(gpio::Level::Low);

            let speaker = pwm::Pwm::new(p.PWM0);
            speaker
                .set_output_pin(pwm::Channel::C0, speaker_pin)
                .set_prescaler(pwm::Prescaler::Div1)
                .set_counter_mode(pwm::CounterMode::Up)
                .set_load_mode(pwm::LoadMode::Common)
                .set_step_mode(pwm::StepMode::Auto)
                .set_max_duty(256)
                .set_seq_refresh(pwm::Seq::Seq0, 0)
                .set_seq_end_delay(pwm::Seq::Seq0, 0)
                .set_seq_refresh(pwm::Seq::Seq1, 0)
                .set_seq_end_delay(pwm::Seq::Seq1, 0)
                .enable_channel(pwm::Channel::C0)
                .enable_group(pwm::Group::G0)
                .loop_inf()
                .enable();

            static mut SAMPS: [u16; 240] = [0; 240];
            sine(&mut SAMPS, 256, 4);
            for s in &mut SAMPS {
                *s |= 0x8000;
            }

            // Start the sine wave.
            let _pwm_seq = speaker.load(Some(&SAMPS), Some(&SAMPS), true).unwrap();
        }
    }

    pub fn speaker_off(&self) {
        unsafe {
            let p = pac::Peripherals::steal();
            let my_pins = Parts::new(p.P0);
            let speaker_pin = my_pins
                .p0_00
                .degrade()
                .into_push_pull_output(gpio::Level::Low);

            let speaker = pwm::Pwm::new(p.PWM0);
            speaker.disable();
        }
    }

    pub fn falling_display(&self) {
        rprintln!("turning on falling display!\n");
    }

    pub fn default_display(&self) {
        // todo
    }
}

pub struct Speaker {
    speaker_pin: gpio::Pin<Output<PushPull>>,
}

pub struct BoardAccel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    sample_size: i32,
    state: BoardState,
}

impl BoardAccel {
    pub fn new() -> BoardAccel {
        Self {
            x: 0,
            y: 0,
            z: 0,
            sample_size: 0,
            state: BoardState::new(),
        }
    }

    pub fn add_to_total(&mut self, x: i32, y: i32, z: i32) {
        self.x = self.x + x;
        self.y = self.y + y;
        self.z = self.z + z;
        self.sample_size = self.sample_size + 1;
    }

    pub fn add_tuple_to_total(&mut self, accel_tuple: (i32, i32, i32)) {
        let (x, y, z): (i32, i32, i32) = accel_tuple;
        self.add_to_total(x, y, z);
    }

    pub fn average_over_sample(&mut self) -> (i32, i32, i32) {
        let (x, y, z) = (
            self.x / self.sample_size,
            self.y / self.sample_size,
            self.z / self.sample_size,
        );
        self.reset();
        (x, y, z)
    }

    pub fn reset(&mut self) {
        self.x = 0;
        self.y = 0;
        self.z = 0;
        self.sample_size = 0;
    }

    pub fn microbit_is_falling(&mut self, x: f32, y: f32, z: f32) {
        let combined_strength = x.powf(2.0) + y.powf(2.0) + z.powf(2.0);
        let result = combined_strength.sqrt() / 1000.0;
        self.state = match self.state {
            BoardState::Falling => {
                if result < 0.4 {
                    rprintln!("still falling! {:?}\n", self.state);
                    self.state
                } else {
                    self.state.next()
                }
            }
            BoardState::NotFalling => {
                if result < 0.5 {
                    self.state.next()
                } else {
                    self.state
                }
            }
        }
    }
}
