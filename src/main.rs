#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprint, rprintln, rtt_init_print};

use cortex_m_rt::entry;
use lsm303agr::Lsm303agr;

use microbit::{
    board::Board,
    display::nonblocking::{BitImage, Display},
    hal::{prelude::*, twim, Timer},
    pac::twim0::frequency::FREQUENCY_A,
    pac::{self, interrupt, TIMER1},
};

use critical_section_lock_mut::LockMut;

use micromath::F32Ext;

static DISPLAY: LockMut<Display<TIMER1>> = LockMut::new();

/*
*/
#[entry]
fn main() -> ! {
    rtt_init_print!();

    // take the peripherals
    let mut board = Board::take().unwrap();
    let mut timer2 = Timer::new(board.TIMER0);

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer2,
            lsm303agr::AccelMode::HighResolution,
            lsm303agr::AccelOutputDataRate::Hz400,
        )
        .unwrap();

    let display = Display::new(board.TIMER1, board.display_pins);
    DISPLAY.init(display);

    unsafe {
        board.NVIC.set_priority(pac::Interrupt::TIMER1, 128);
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
    }

    loop {
        // rprintln!("test");
        let accel_reading = sensor.acceleration().unwrap();
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut z: f32 = 0.0;
        let num_samples: i16 = 100;
        for _ in 0..num_samples {
            x += ((accel_reading.x_mg() as f32) / 1000.0) as f32;
            y += ((accel_reading.y_mg() as f32) / 1000.0) as f32;
            z += ((accel_reading.z_mg() as f32) / 1000.0) as f32;
            timer2.delay_us(200u32);
        }
        (x, y, z) = average_over_sample(num_samples, x, y, z);
        if microbit_is_falling(x, y, z) {
            rprintln!(
                "{} {} {}\t",
                accel_reading.x_mg(),
                accel_reading.y_mg(),
                accel_reading.z_mg()
            );
            rprintln!("Microbit is falling!\n");
        }
        // timer2.delay_ms(10u32);
    }
}

#[interrupt]
fn TIMER1() {
    DISPLAY.with_lock(|display| display.handle_display_event());
}

fn microbit_is_falling(x: f32, y: f32, z: f32) -> bool {
    let combined_strength = x.powf(2.0) + y.powf(2.0) + z.powf(2.0);
    let result = combined_strength.sqrt();
    if result < 0.3 {
        rprintln!("result is less than .3?: {}\n", result);
        rprintln!("{} {} {}\t", x, y, z);
        true
    } else {
        false
    }
}

fn average_over_sample(sample_size: i16, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let divisor: f32 = sample_size.into();
    (x / divisor, y / divisor, z / divisor)
}
