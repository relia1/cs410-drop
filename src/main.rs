#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m::asm;
use cortex_m_rt::entry;

use drop::{BoardAccel, BoardState}; // {BoardState, GPIO, MB2};
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

// pub static GPIO: LockMut<Gpiote> = LockMut::new();
static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board_accel = BoardAccel::new();

    let mut board = Board::take().unwrap();
    let display = Display::new(board.TIMER1, board.display_pins);
    DISPLAY.init(display);

    let mut timer = Timer::new(board.TIMER0);
    let state = BoardState::NotFalling;
    let i2c = twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100);
    let mut sensor = Lsm303agr::new_with_i2c(i2c);

    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer,
            lsm303agr::AccelMode::HighResolution,
            lsm303agr::AccelOutputDataRate::Hz100,
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
        if counter >= 10 {
            let (x, y, z) = board_accel.average_over_sample();
            rprintln!("Averages x: {} y: {} z: {}", x, y, z);
            board_accel.microbit_is_falling(x as f32, y as f32, z as f32);
            counter = 0;
        }
        if sensor.accel_status().unwrap().xyz_new_data() {
            board_accel.add_tuple_to_total(sensor.acceleration().unwrap().xyz_mg());
            counter += 1;
            // rprintln!("not waiting\n");
        } else {
            // rprintln!("waiting\n");
            asm::wfi();
        }
        /*
        MB2_ACCEL.with_lock(|cs| {
            let (a, b, c) = cs.get_accel_data();
            rprintln!("{} {} {}\n", a, b, c);
        });
        */

        //rprintln!("main loop accel int gpiote");
        /*
        let mut x: i32 = 0.0;
        let mut y: i32 = 0.0;
        let mut z: i32 = 0.0;
        let num_samples: i16 = 20;
        for _ in 0..num_samples {
            let data = mb2_board.get_accel_data();
            x += data.0;
            y += data.1;
            z += data.2;
        }
        (x, y, z) = average_over_sample(num_samples, x, y, z);
        match mb2_board.state {
            BoardState::Falling => {
                rprintln!("{} {} {}\t", x, y, z);
                rprintln!("Microbit is falling!\n");
            }
            BoardState::NotFalling => {}
        }
        */
        /*
        critical_section::with(|cs| {
            let (a, b, c) = mb2_board.get_accel_data();
            rprintln!("{} {} {}\t", a, b, c);
        });
        */
    }
}

/*
fn microbit_is_falling(x: i32, y: i32, z: i32) -> BoardState {
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
*/

/*
fn average_over_sample(sample_size: i16, x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    // rprintln!("{} {} {}\t", x * 1000.0, y * 1000.0, z * 1000.0);
    let divisor: i32 = sample_size.into();
    (x / divisor, y / divisor, z / divisor)
}
*/

/*
 * concept: have an interrupt for the imu (accelerometer) that fills a queue and when
 * that queue is filled take a look at the data
*/
#[interrupt]
fn TIMER1() {
    // rprintln!("timer int");
    DISPLAY.with_lock(|display| display.handle_display_event());
}
