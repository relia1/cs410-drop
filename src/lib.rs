#![no_std]
// my own custom board abstractions go here

use panic_rtt_target as _;

use crate::twim::Twim;
use core::convert::Into;
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagOneShot;
use lsm303agr::{Interrupt, Lsm303agr};
use rtt_target::rprintln;

use microbit::{
    board::{Board, I2CInternalPins},
    display::nonblocking::Display,
    hal::gpiote::{Gpiote, GpioteChannel},
    hal::{
        gpio::{self, p0, Disconnected, Floating, Input, Pin},
        twim, Timer,
    },
    pac::twim0::frequency::FREQUENCY_A,
    pac::{self as pac, interrupt, TIMER0, TIMER1},
};

use critical_section_lock_mut::LockMut;
use micromath::F32Ext;
use pac::P0;

static GPIO: LockMut<Gpiote> = LockMut::new();
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
    pub fn new() -> Result<Self, &'static str> {
        if let Some(board) = Board::take() {
            let gpiote = Gpiote::new(board.GPIOTE);
            let mut timer = Timer::new(board.TIMER0);
            let i2c_pins: twim::Pins = board.i2c_internal.into();
            let i2c = { twim::Twim::new(board.TWIM0, i2c_pins, FREQUENCY_A::K100) };
            let mut sensor = Lsm303agr::new_with_i2c(i2c);

            sensor.init().unwrap();
            sensor
                .set_accel_mode_and_odr(
                    &mut timer,
                    lsm303agr::AccelMode::HighResolution,
                    lsm303agr::AccelOutputDataRate::Hz100,
                )
                .unwrap();

            sensor.acc_enable_interrupt(Interrupt::DataReady1).unwrap();
            let state = BoardState::NotFalling;

            let display = Display::new(board.TIMER1, board.display_pins);

            DISPLAY.init(display);

            let setup_channel = |channel: GpioteChannel, pin: gpio::Pin<Input<Floating>>| {
                channel.input_pin(&pin).hi_to_lo().enable_interrupt();
                channel.reset_events();
            };

            setup_channel(
                gpiote.channel0(),
                board.pins.p0_13.into_floating_input().degrade(),
            );

            GPIO.init(gpiote);

            unsafe {
                //            board.NVIC.set_priority(pac::Interrupt::TIMER1, 128);
                //            pac::NVIC::unmask(pac::Interrupt::TIMER1);
                //board.NVIC.set_priority(pac::Interrupt::GPIOTE, 10);
                pac::NVIC::unmask(pac::Interrupt::GPIOTE);
            }
            pac::NVIC::unpend(pac::Interrupt::GPIOTE);

            Ok(MB2 {
                sensor,
                timer,
                state,
            })
        } else {
            Err("Board not available")
        }
        // rprintln!("{:?}\n", sensor.accel_status());
    }

    pub fn get_accel_data(&mut self) -> (f32, f32, f32) {
        let accel_reading = self.sensor.acceleration().unwrap();
        // rprintln!("{:?}\n", self.sensor.accel_status());
        let (x, y, z) = accel_reading.xyz_mg();
        (
            (x as f32) / 1000.0,
            (y as f32) / 1000.0,
            (z as f32) / 1000.0,
        )
    }
}

/*
#[interrupt]
fn TIMER1() {
    DISPLAY.with_lock(|display| display.handle_display_event());
}

#[interrupt]
fn TIMER0() {}
*/

fn microbit_is_falling(x: f32, y: f32, z: f32) -> BoardState {
    let combined_strength = x.powf(2.0) + y.powf(2.0) + z.powf(2.0);
    let result = combined_strength.sqrt();
    if result < 0.27 {
        rprintln!("result is less than .3?: {}\n", result);
        rprintln!("{} {} {}\t", x, y, z);
        BoardState::Falling
    } else {
        BoardState::NotFalling
    }
}

#[interrupt]
fn GPIOTE() {
    rprintln!("accel int");
    GPIO.with_lock(|gpiote| {
        rprintln!("accel int");
    });
}

#[interrupt]
fn SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0() {
    rprintln!("accel int");
}
