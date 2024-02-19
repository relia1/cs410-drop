#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;

use drop::{BoardState, MB2};

use microbit::{hal::prelude::*};



/*
*/
#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut mb2_board = MB2::new().unwrap();

    loop {
        // rprintln!("test");
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut z: f32 = 0.0;
        let num_samples: i16 = 20;
        for _ in 0..num_samples {
            let data = mb2_board.get_accel_data();
            x += data.0;
            y += data.1;
            z += data.2;

            // rprintln!("print\t{}, {}, {}\t", x * 1000.0, y * 1000.0, z * 1000.0);
            mb2_board.timer.delay_us(500u32);
        }
        (x, y, z) = average_over_sample(num_samples, x, y, z);
        // rprintln!("{} {} {}\t", x * 1000.0, y * 1000.0, z * 1000.0);
        // let state = microbit_is_falling(x, y, z);
        match mb2_board.state {
            BoardState::Falling => {
                rprintln!("{} {} {}\t", x, y, z);
                rprintln!("Microbit is falling!\n");
            }
            BoardState::NotFalling => {}
        }
        // timer2.delay_ms(10u32);
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
