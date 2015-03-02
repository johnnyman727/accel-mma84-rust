#![feature(old_io, old_path, env)]
#![allow(dead_code)]
mod tessel;
use tessel::TesselPort;
use tessel::Action;

const VALID_CHIP_ID: u8 = 42;
const I2C_ADDRESS: u8 = 0x1D;
const OUT_X_MSB: u8 = 0x01;
const CTRL_REG1: u8 = 0x2a;
const CHIP_ID_REG: u8 = 0x0D;


pub struct Accelerometer {
  port: TesselPort,
}

impl Accelerometer {
  pub fn new(p: TesselPort) -> Accelerometer {
    let mut accelerometer = Accelerometer {
      port : p,
    };

    match accelerometer.get_chip_id() {
      Ok(VALID_CHIP_ID) => accelerometer,
      Err(e) => panic!(e),
      _ => panic!("Unable to read chip id."),
    }
  }

  pub fn get_chip_id(&mut self) -> Result<u8, &'static str>  {
    let mut chip_id = [0]; 

    let res = self.port.run(&mut [
      // Enable I2C
      Action::enable_i2c(),
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[CHIP_ID_REG]),
      Action::start(0x1 | I2C_ADDRESS << 1),
      Action::rx(&mut chip_id),
      Action::stop(),
    ]);

    println!("Debug: got chip id {:?}", chip_id[0]);
    // I think it might be cleaner to also result an IOResult<u8, Error>
    // with the same error returned from running these actions on the port
    match res {
      Ok(v) => Ok(chip_id[0]),
      Err(e) => Err("Unable to read chip ID."),
    }
  }

  pub fn mode_active(&mut self) -> () {
    let mut reg_current_state = [0];
    let active_bit: u8 = 0x01;

    self.port.run(&mut [
      // Enable I2C
      Action::enable_i2c(),
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[CTRL_REG1]),
      Action::start(0x1 | I2C_ADDRESS << 1),
      Action::rx(&mut reg_current_state),
      Action::stop(),
    ]).unwrap();

    self.port.run(&mut [
      // Enable I2C
      Action::enable_i2c(),
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[CTRL_REG1, reg_current_state[0] | active_bit]),
      Action::stop(),
    ]).unwrap();
  }

  pub fn get_acceleration(&mut self, accel: &mut[u16]) -> () {
    let mut accel_raw = [0;6];
    let mut out = [0;3];

    self.port.run(&mut [
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[OUT_X_MSB]),
      Action::start(0x1 | I2C_ADDRESS << 1),
      Action::rx(&mut accel_raw),
      Action::stop(),
    ]).unwrap();

    let mut i = 0;
    while i < 3 {
      let mut g_count:u16 = (((accel_raw[i*2] as u16) << 8) | (accel_raw[(i*2)+1]) as u16) as u16;
      g_count = g_count >> 4;

      // If the number is negative, we have to make it so manually (no 12-bit data type)
      if accel_raw[i*2] > 0x7F {
        g_count = -(1 + 0xFFF - g_count); // Transform into negative 2's complement
      }

      g_count / ((1<<12)/(2*2/*self.scaleRange*/));

      accel[i] = g_count;

      i= i + 1;
    }
  }
}

fn main() {
  let args = std::env::args().collect::<Vec<_>>();
  let p = tessel::TesselPort::new(&Path::new(&args[1]));
  let mut accel = Accelerometer::new(p);
  let mut accel_values = [0;3];
  accel.mode_active();
  accel.get_acceleration(&mut accel_values);
  
  println!("x: {:?}, y: {:?}, z: {:?}", accel_values[0], accel_values[1], accel_values[2]);
  
}