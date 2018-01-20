extern crate backlight;
extern crate tini;

use std::fs::OpenOptions;
use std::fs::{remove_file, File};
use std::io::prelude::*;
use std::{thread, time};
use std::io::SeekFrom;
use std::fs::create_dir_all;
use std::cmp;
use std::env;
use std::path::Path;
use std::process;
use tini::Ini;
use backlight::Backlight;

const LUMOS_LOCK: &'static str = "/tmp/lumos.lock";

extern "C" {
    fn signal(sig: u32, cb: extern "C" fn(u32)) -> extern "C" fn(u32);
}

extern "C" fn interrupt(signal: u32) {
    println!("[!] Lumos interrupted by signal {}!", signal);
    remove_file(LUMOS_LOCK).unwrap();
    process::exit(0);
}

#[derive(Debug)]
struct Illuminance {
    file: File,
}

impl Illuminance {
    fn from_config(config: &Ini) -> Illuminance {
        let filename: String = config.get("illuminance", "file").unwrap();
        let file = File::open(filename).unwrap();
        Illuminance { file }
    }

    fn get(&mut self) -> i32 {
        let mut buffer = String::new();
        self.file.seek(SeekFrom::Start(0)).unwrap();
        self.file.read_to_string(&mut buffer).unwrap();
        match buffer.trim().parse::<i32>() {
            Ok(value) => value,
            Err(_) => panic!("can't parse `{}` value", buffer),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Transition {
    step: time::Duration,
    sleep: time::Duration,
    start: f32,
    end: f32,
    steps: i32,
    cur: i32,
}

impl Transition {
    fn from_config(config: &Ini) -> Transition {
        let step = time::Duration::from_millis(config.get("transition", "step").unwrap());
        let sleep = time::Duration::from_millis(config.get("transition", "sleep").unwrap());
        Transition {
            step,
            sleep,
            start: 0_f32,
            end: 0_f32,
            steps: 0_i32,
            cur: 0_i32,
        }
    }

    /// Transition function. f(0) = 0, f(1) = 1
    fn f(x: f32) -> f32 {
        1.0 / ((-15.0 * (x - 0.5)).exp() + 1.0)
    }

    /// Current progress of transition
    fn progress(&self) -> f32 {
        Transition::f(self.cur as f32 / self.steps as f32)
    }

    pub fn set(&mut self, start: f32, end: f32) {
        self.start = start;
        self.end = end;
        self.steps = if (self.end - self.start).abs() > 0.1 {
            cmp::min(30, ((self.end - self.start).abs() * 100f32) as i32)
        } else {
            0i32
        };
        self.cur = 0i32;
    }
}

impl Iterator for Transition {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.cur >= self.steps - 1 {
            thread::sleep(self.sleep);
            return None;
        }
        thread::sleep(self.step);
        let v = self.start + (self.end - self.start) * self.progress();
        self.cur += 1;
        Some(v)
    }
}

struct Transform {
    i2b: Vec<i32>,
}

impl Transform {
    fn from_config(config: &Ini) -> Transform {
        let i2b: Vec<i32> = config.get_vec("illuminance", "i2b").unwrap();
        Transform { i2b }
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
            (r as f32 + (value - self.i2b[r]) as f32 / (self.i2b[r] - self.i2b[r - 1]) as f32)
                * step
        }
    }
}

fn create_default_config() -> Ini {
    Ini::new()
        .section("illuminance")
        .item(
            "file",
            "/sys/bus/acpi/devices/ACPI0008:00/iio:device0/in_illuminance_raw",
        )
        .item("i2b", "-5,20,300,700,1100,7100")
        .section("transition")
        .item("step", "50")
        .item("sleep", "1000")
}

fn main() {
    // ignore multiple copy of app
    if Path::new(LUMOS_LOCK).exists() {
        println!(
            "[!] Lumos is already run!\n\
            - you can't run more than one copy of lumos\n\
            - maybe last session was ended incorrectly (remove {})",
            LUMOS_LOCK
        );
        process::exit(0);
    }
    // create lock file
    let _ = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(LUMOS_LOCK)
        .unwrap();
    // interrupt signals
    unsafe {
        signal(1, interrupt);
        signal(2, interrupt);
        signal(9, interrupt);
        signal(15, interrupt);
    }

    // user configs
    let user_home = env::var("HOME").unwrap();
    let user_path = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => Path::new(&path).join("lumos/config.ini"),
        Err(_) => Path::new(&user_home).join(".config/lumos/config.ini"),
    };
    if !user_path.exists() {
        let default_config = create_default_config();
        create_dir_all(user_path.parent().unwrap()).unwrap();
        default_config.to_file(&user_path).unwrap();
    }

    // debug log
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/lumos.log")
        .unwrap();

    // load config
    let config = Ini::from_file(&user_path).unwrap();
    let blinking_flag: bool = config.get("debug", "fix_blinking").unwrap_or(false);
    let debug_log: bool = config.get("debug", "log").unwrap_or(false);

    let backlight = Backlight::new();
    let mut illuminance = Illuminance::from_config(&config);
    let transform = Transform::from_config(&config);
    let mut transition = Transition::from_config(&config);
    loop {
        // dirty hack for avoid blinking
        let mut value = illuminance.get();
        if value == 0 {
            // maybe sleep here?
            value = illuminance.get();
        }
        transition.set(backlight.get(), transform.to_backlight(value));
        for v in transition {
            backlight.set(v);
        }
        if blinking_flag {
            thread::sleep(time::Duration::from_millis(50));
        }
        if debug_log {
            write!(log, "{} ", value).ok();
        }
    }
}
