extern crate tini;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{thread, time};
use std::io::SeekFrom;
use std::cmp;
use tini::Ini;

#[derive(Debug)]
struct Configuration {
    max_backlight: i32,
    min_backlight: i32,
    min_illuminance: i32,
    max_illuminance: i32,
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

    fn set(&mut self, value: i32) {
        if value >= self.min_backlight && value <= self.max_backlight as i32 {
            self.backlight.seek(SeekFrom::Start(0)).ok();
            write!(self.backlight, "{}", value).ok();
        }
    }

    fn get(&mut self) -> i32 {
        let mut buffer = String::new();
        self.illuminance.seek(SeekFrom::Start(0)).ok();
        self.illuminance.read_to_string(&mut buffer).ok();
        match buffer.trim().parse::<i32>() {
            Ok(value) => value,
            Err(_) => panic!("can't parse `{}` value", buffer)
        }
    }

    fn lumos_to_backlight(&self, v: i32) -> i32 {
        let x = (v - self.min_illuminance) as f32 / (self.max_illuminance - self.min_illuminance) as f32;
        (self.min_backlight as f32 + (self.max_backlight - self.min_backlight) as f32 * ((2f32 - x) * x).sqrt()) as i32
    }
}

fn transition(x: f32, center: f32, range: f32) -> f32 {
    1.0 / ((15.0 * (x - center) / range).exp() + 1.0)
}

fn main() {
    let sleep_time = time::Duration::from_millis(100);
    let step_time = time::Duration::from_millis(50);
    let mut config = Configuration::init("config.ini");
    let mut start = 0;
    let mut end = 0;
    loop {
        start = end;
        end = config.get();
        let mut steps = cmp::min(30, (end - start).abs() / 10);
        println!("steps = {}", steps);
        if steps > 0 {
            steps += 5;
            for i in 0..steps + 1 {
                let v = ((start - end) as f32 * transition(i as f32, steps as f32 / 2.0, steps as f32)) as i32 + end;
                println!("v = {}", v);
                let bv = config.lumos_to_backlight(v);
                println!("bv = {}", bv);
                config.set(bv);
                thread::sleep(step_time);
            }
        } else {
            let value = config.lumos_to_backlight(end);
            config.set(value);
        }
        thread::sleep(sleep_time);
    }
}
