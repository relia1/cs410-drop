#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m::asm;
use cortex_m_rt::entry;

use drop::{BoardState, GPIO, MB2};
use microbit::pac::{self as pac, interrupt, TIMER0, TIMER1};

// use microbit::hal::prelude::*;
use critical_section_lock_mut::critical_section;
use critical_section_lock_mut::LockMut;

static MB2_ACCEL: LockMut<MB2> = LockMut::new();
/*
*/
#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut mb2_board = MB2::new().unwrap();
    mb2_board.get_accel_data();
    MB2_ACCEL.init(mb2_board);
    rprintln!("before loop main accel int gpiote");
    // pac::NVIC::unpend(pac::Interrupt::GPIOTE);

    /*
    MB2_ACCEL.with_lock(|cs| {
        cs.get_accel_data();
    });
    */
    loop {
        unsafe {
            pac::NVIC::unmask(pac::Interrupt::GPIOTE);
        }
        rprintln!("before loop main accel int gpiote");

        pac::NVIC::unpend(pac::Interrupt::GPIOTE);

        /*
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut z: f32 = 0.0;
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
        asm::wfi();
        critical_section::with(|cs| {
            let (a, b, c) = mb2_board.get_accel_data();
            rprintln!("{} {} {}\t", a, b, c);
        });
    }
}

/*
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
*/

fn average_over_sample(sample_size: i16, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    // rprintln!("{} {} {}\t", x * 1000.0, y * 1000.0, z * 1000.0);
    let divisor: f32 = sample_size.into();
    (x / divisor, y / divisor, z / divisor)
}

/*
 * concept: have an interrupt for the imu (accelerometer) that fills a queue and when
 * that queue is filled take a look at the data
*/

#[interrupt]
fn GPIOTE() {
    GPIO.with_lock(|gpiote| {
        // let (a, b, c) = mb2_accel.get_accel_data();
        // rprintln!("accel int gpiote: {} {} {}", a, b, c);
        rprintln!("interrupt {}\n", gpiote.channel0().is_event_triggered());
    });
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);
}
