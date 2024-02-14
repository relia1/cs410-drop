#![no_std]
// my own custom board abstractions go here

use panic_rtt_target as _;

use crate::twim::Twim;
use core::convert::Into;
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagOneShot;
use lsm303agr::Lsm303agr;

use microbit::{
    board::Board,
    display::nonblocking::Display,
    hal::{twim, Timer},
    pac::twim0::frequency::FREQUENCY_A,
    pac::{self, interrupt, TIMER0, TIMER1},
};

use critical_section_lock_mut::LockMut;

static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();

pub enum BoardState {
    Falling,
    NotFalling,
}

pub struct MB2 {
    pub sensor: Lsm303agr<I2cInterface<Twim<pac::TWIM0>>, MagOneShot>,
    pub timer: Timer<TIMER0>,
    pub state: BoardState,
}

impl MB2 {
    pub fn new(mut board: Board) -> Self {
        let mut timer = Timer::new(board.TIMER0);

        let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
        let mut sensor: Lsm303agr<I2cInterface<Twim<pac::TWIM0>>, MagOneShot> =
            Lsm303agr::new_with_i2c(i2c);
        sensor.init().unwrap();
        sensor
            .set_accel_mode_and_odr(
                &mut timer,
                lsm303agr::AccelMode::HighResolution,
                lsm303agr::AccelOutputDataRate::Hz400,
            )
            .unwrap();

        let state = BoardState::NotFalling;

        let display = Display::new(board.TIMER1, board.display_pins);

        DISPLAY.init(display);
        unsafe {
            board.NVIC.set_priority(pac::Interrupt::TIMER1, 128);
            pac::NVIC::unmask(pac::Interrupt::TIMER1);
        }

        MB2 {
            sensor,
            timer,
            state,
        }
    }

    pub fn get_accel_data(&mut self) -> (f32, f32, f32) {
        let accel_reading = self.sensor.acceleration().unwrap();
        let (x, y, z) = accel_reading.xyz_mg();
        (
            (x as f32) / 1000.0,
            (y as f32) / 1000.0,
            (z as f32) / 1000.0,
        )
    }
}

#[interrupt]
fn TIMER1() {
    DISPLAY.with_lock(|display| display.handle_display_event());
}
