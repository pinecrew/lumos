extern crate tini;
extern crate backlight;

use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use std::io::SeekFrom;
use std::fs::create_dir_all;
use std::cmp;
use std::env;
use std::path::Path;
use tini::Ini;
use backlight::Backlight;

#[derive(Debug)]
struct Illuminance {
    file: File,
}


impl Illuminance {
    fn from_config(config: &Ini) -> Illuminance {
        let filename: String = config.get("illuminance", "file").unwrap();
        let file = File::open(filename).unwrap();
        Illuminance {
            file,
        }
    }

    fn get(&mut self) -> i32 {
        let mut buffer = String::new();
        self.file.seek(SeekFrom::Start(0)).ok();
        self.file.read_to_string(&mut buffer).ok();
        match buffer.trim().parse::<i32>() {
            Ok(value) => value,
            Err(_) => panic!("can't parse `{}` value", buffer)
        }
    }
}

#[derive(Debug)]
struct Transition {
    step: time::Duration,
    sleep: time::Duration,
}

impl Transition {
    fn from_config(config: &Ini) -> Transition {
        let step = time::Duration::from_millis(config.get("transition", "step").unwrap());
        let sleep = time::Duration::from_millis(config.get("transition", "sleep").unwrap());
        Transition {
            step,
            sleep,
        }
    }
    pub fn f(&self, x: f32, center: f32, range: f32) -> f32 {
        1.0 / ((15.0 * (x - center) / range).exp() + 1.0)
    }
}

struct Transform {
    i2b: Vec<i32>,
}

impl Transform {
    fn from_config(config: &Ini) -> Transform {
        let i2b: Vec<i32> = config.get_vec("illuminance", "i2b").unwrap();
        Transform {
            i2b,
        }
    }

    pub fn to_backlight(&self, value: i32) -> f32 {
        let last = self.i2b.len() - 1;
        let step = 1f32 / last as f32;
        let mut r = last + 1;
        for (n, i) in self.i2b.iter().enumerate() {
            if value < *i {
                r = n;
                break;
            }
        }
        if r == 0 {
            0f32
        } else if r > last {
            1f32
        } else {
            (r as f32 +
               (value - self.i2b[r]) as f32 /
               (self.i2b[r] - self.i2b[r-1]) as f32) * step
        }
    }
}

fn create_default_config() -> Ini {
    Ini::new()
    .section("illuminance")
        .item("file", "/sys/bus/acpi/devices/ACPI0008:00/iio:device0/in_illuminance_raw")
        .item("i2b", "-5,20,300,700,1100,7100")
    .section("transition")
        .item("step", "50")
        .item("sleep", "1000")
}

fn main() {
    let default_config = create_default_config();
    let user_home = env::var("HOME").unwrap();
    let user_path = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => Path::new(&path).join("lumos/config.ini"),
        Err(_) => Path::new(&user_home).join(".config/lumos/config.ini"),
    };
    if !user_path.exists() {
        create_dir_all(user_path.parent().unwrap()).unwrap();
        default_config.to_file(&user_path).unwrap();
    }
    let config = Ini::from_file(&user_path).unwrap();
    let backlight = Backlight::new();
    let mut illuminance = Illuminance::from_config(&config);
    let transform = Transform::from_config(&config);
    let transition = Transition::from_config(&config);
    let mut end = backlight.get();
    loop {
        let start = end;
        end = transform.to_backlight(illuminance.get());
        let mut steps = cmp::min(30, ((end - start).abs() * 100f32 ) as i32);
        if steps > 0 {
            steps += 5;
            for i in 0..steps + 1 {
                let v = ((start - end) * transition.f(i as f32, steps as f32 / 2.0, steps as f32)) + end;
                backlight.set(v);
                thread::sleep(transition.step);
            }
        } else {
            backlight.set(end);
        }
        thread::sleep(transition.sleep);
    }
}
