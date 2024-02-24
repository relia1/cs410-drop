#![no_std]
// my own custom board abstractions go here

use micromath::F32Ext;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

pub enum BoardState {
    Falling,
    NotFalling,
}

pub struct BoardAccel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    sample_size: i32,
}

impl BoardAccel {
    pub fn new() -> BoardAccel {
        Self {
            x: 0,
            y: 0,
            z: 0,
            sample_size: 0,
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

    pub fn microbit_is_falling(&self, x: f32, y: f32, z: f32) -> BoardState {
        let combined_strength = x.powf(2.0) + y.powf(2.0) + z.powf(2.0);
        let result = combined_strength.sqrt();
        if (result / 1000.0) < 0.35 {
            rprintln!("result is less than .3?: {}\n", result);
            rprintln!("{} {} {}\t", x, y, z);
            BoardState::Falling
        } else {
            rprintln!("result is more than .3?: {}\n", result);
            BoardState::NotFalling
        }
    }
}

/*
pub struct MB2 {
    pub sensor: Lsm303agr<I2cInterface<Twim<pac::TWIM0>>, MagOneShot>,
    pub timer: Timer<TIMER0>,
    pub state: BoardState,
}
*/

/*
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
*/
