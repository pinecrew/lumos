extern crate tini;
extern crate backlight;

use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use std::io::SeekFrom;
use std::cmp;
use tini::Ini;
use backlight::Backlight;

#[derive(Debug)]
struct Configuration {
    max_backlight: u8,
    min_backlight: u8,
    min_illuminance: i32,
    max_illuminance: i32,
    backlight: Backlight,
    illuminance: File,
    transition_step: time::Duration,
    transition_sleep: time::Duration,
}

impl Configuration {
    fn init(filename: &str) -> Configuration {
        let config = Ini::from_file(filename).unwrap();

        let max_backlight = config.get("backlight", "max").unwrap();
        let min_backlight = config.get("backlight", "min").unwrap();
        let max_illuminance = config.get("illuminance", "max").unwrap();
        let min_illuminance = config.get("illuminance", "min").unwrap();
        let backlight = Backlight::init();
        let illuminance_file: String = config.get("config", "illuminance_file").unwrap();
        let illuminance = File::open(illuminance_file).unwrap();
        let transition_step = time::Duration::from_millis(config.get("transition", "step").unwrap());
        let transition_sleep = time::Duration::from_millis(config.get("transition", "sleep").unwrap());

        Configuration {
            max_backlight,
            min_backlight,
            max_illuminance,
            min_illuminance,
            backlight,
            illuminance,
            transition_step,
            transition_sleep,
        }
    }

    fn set(&mut self, value: u8) {
        self.backlight.set(value);
        // hack
        let _ = self.backlight.get();
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

    fn lumos_to_backlight(&self, v: i32) -> u8 {
        let x = (v - self.min_illuminance) as f32 / (self.max_illuminance - self.min_illuminance) as f32;
        (self.min_backlight as f32 + (self.max_backlight - self.min_backlight) as f32 * ((2f32 - x) * x).sqrt()) as u8
    }
}

fn transition(x: f32, center: f32, range: f32) -> f32 {
    1.0 / ((15.0 * (x - center) / range).exp() + 1.0)
}

fn main() {
    let mut config = Configuration::init("config.ini");
    let mut end = 0;
    loop {
        let start = end;
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
                thread::sleep(config.transition_step);
            }
        } else {
            let value = config.lumos_to_backlight(end);
            config.set(value);
        }
        thread::sleep(config.transition_sleep);
    }
}
