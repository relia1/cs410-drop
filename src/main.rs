#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m::asm;
use cortex_m_rt::entry;

use drop::{BoardAccel, BoardState, DISPLAY}; // {BoardState, GPIO, MB2};
use microbit::pac::{self as pac, interrupt};
use microbit::{
    board::Board,
    display::nonblocking::Display,
    hal::{twim, Timer},
    pac::twim0::frequency::FREQUENCY_A,
    pac::TIMER1,
};

use lsm303agr::{AccelScale, Lsm303agr};

use critical_section_lock_mut::LockMut;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board_accel = BoardAccel::new();

    let mut board = Board::take().unwrap();
    // BOARD.init(board);
    let display = Display::new(board.TIMER1, board.display_pins);
    DISPLAY.init(display);

    let mut timer = Timer::new(board.TIMER0);
    let i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
    let mut sensor = Lsm303agr::new_with_i2c(i2c);

    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer,
            lsm303agr::AccelMode::HighResolution,
            lsm303agr::AccelOutputDataRate::Hz200,
        )
        .unwrap();
    sensor.set_accel_scale(AccelScale::G2).unwrap();

    unsafe {
        board.NVIC.set_priority(pac::Interrupt::TIMER1, 6);
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
    }

    pac::NVIC::unpend(pac::Interrupt::TIMER1);

    let mut counter: u8 = 0;
    loop {
        if counter >= 5 {
            let (x, y, z) = board_accel.average_over_sample();
            board_accel.microbit_is_falling(x as f32, y as f32, z as f32);
            counter = 0;
        }

        if sensor.accel_status().unwrap().xyz_new_data() {
            board_accel.add_tuple_to_total(sensor.acceleration().unwrap().xyz_mg());
            counter += 1;
        }
        asm::wfi();
    }
}

#[interrupt]
fn TIMER1() {
    DISPLAY.with_lock(|display| display.handle_display_event());
}
