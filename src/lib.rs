#![no_std]

use critical_section_lock_mut::LockMut;
use microbit::{
    display::nonblocking::{BitImage, Display},
    hal::{
        gpio::{self, p0::Parts},
        pwm,
    },
    pac::{self, TIMER1},
};

use micromath::F32Ext;
use panic_rtt_target as _;

pub static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();
pub static SPEAKER: LockMut<BoardState> = LockMut::new();

#[derive(Debug, Clone, Copy)]
pub enum BoardState {
    Falling,
    NotFalling,
}

impl BoardState {
    pub fn new() -> Self {
        let board_state = { Self::NotFalling };
        board_state.default_display();
        board_state
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
        unsafe {
            let p = pac::Peripherals::steal();
            let my_pins = Parts::new(p.P0);
            let speaker_pin = my_pins
                .p0_00
                .degrade()
                .into_push_pull_output(gpio::Level::Low);

            // https://github.com/pdx-cs-rust-embedded/mb2-audio-experiments/blob/hw-pwm/src/main.rs
            // https://github.com/pdx-cs-rust-embedded/hello-audio/blob/main/src/main.rs
            // referenced as examples
            let speaker = pwm::Pwm::new(p.PWM0);
            speaker
                .set_output_pin(pwm::Channel::C0, speaker_pin)
                .set_prescaler(pwm::Prescaler::Div1)
                .set_counter_mode(pwm::CounterMode::Up)
                .set_load_mode(pwm::LoadMode::Common)
                .set_step_mode(pwm::StepMode::Auto)
                .set_max_duty(128)
                .enable_channel(pwm::Channel::C0)
                .enable_group(pwm::Group::G0)
                .loop_inf()
                .enable();

            static mut SQUARE_WAVE0: [u16; 64] = [0; 64];
            static mut SQUARE_WAVE1: [u16; 64] = [0; 64];
            for i in 0..64 {
                SQUARE_WAVE0[i] = 0x8000;
            }

            for i in 0..64 {
                SQUARE_WAVE1[i] = 0x0000;
            }

            // Start the square wave
            let _pwm_seq = speaker
                .load(Some(&SQUARE_WAVE0), Some(&SQUARE_WAVE1), true)
                .unwrap();
        }
    }

    pub fn speaker_off(&self) {
        unsafe {
            let p = pac::Peripherals::steal();
            let speaker = pwm::Pwm::new(p.PWM0);
            speaker.disable();
        }
    }

    pub fn falling_display(&self) {
        let image: [[u8; 5]; 5] = [
            [0, 0, 1, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
        ];
        DISPLAY.with_lock(|display| display.show(&BitImage::new(&image)));
    }

    pub fn default_display(&self) {
        let image: [[u8; 5]; 5] = [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ];
        DISPLAY.with_lock(|display| display.show(&BitImage::new(&image)));
    }
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
                if result < 0.55 {
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
