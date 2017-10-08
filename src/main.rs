extern crate tini;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{thread, time};
use std::mem;
use tini::Ini;

#[derive(Debug)]
struct Configuration {
    max_backlight: i32,
    min_backlight: u32,
    min_illuminance: u32,
    max_illuminance: u32,
    backlight: File,
    illuminance: File,
}

impl Configuration {
    fn init(filename: &str) -> Configuration {
        let config = Ini::from_file(filename).unwrap();

        let max_backlight = match config.get("backlight", "max") {
            Some(-1) => {
                let max_backlight_file: String = config.get("config", "max_backlight_file").unwrap();
                let mut file = File::open(max_backlight_file).unwrap();
                let mut buffer = String::new();
                file.read_to_string(&mut buffer).ok();
                buffer.trim_right().parse::<i32>().unwrap()
            }
            Some(value) => value,
            None => panic!("ахтунг!")
        };
        let min_backlight = config.get("backlight", "min").unwrap();
        let max_illuminance = config.get("illuminance", "max").unwrap();
        let min_illuminance = config.get("illuminance", "min").unwrap();
        let backlight_file: String = config.get("config", "backlight_file").unwrap();
        let backlight = OpenOptions::new()
                                    .write(true)
                                    .open(backlight_file)
                                    .unwrap();
        let illuminance_file: String = config.get("config", "illuminance_file").unwrap();
        let illuminance = File::open(illuminance_file).unwrap();

        Configuration {
            max_backlight,
            min_backlight,
            max_illuminance,
            min_illuminance,
            backlight,
            illuminance
        }
    }

    fn set(&mut self, value: u32) {
        if value >= self.min_backlight && value <= self.max_backlight as u32 {
            write!(self.backlight, "{}", value);
        }
    }

    fn get(&mut self) -> u32 {
        let mut buffer = [0; 4];
        self.illuminance.read(&mut buffer[..]);
        unsafe {
            mem::transmute::<[u8; 4], u32>(buffer)
        }
    }
}

fn main() {
    let sleep_time = time::Duration::from_millis(100);
    let mut config = Configuration::init("config.ini");
    loop {
        let val = config.get();
        println!("val = {}", val);
        thread::sleep(sleep_time);
    }
}
