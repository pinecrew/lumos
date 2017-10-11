extern { 
    fn backlight_init();
    fn backlight_get() -> i32; 
    fn backlight_set(value: i32);
    fn backlight_max() -> i32;
    fn backlight_min() -> i32;
}

#[derive(Debug)]
pub struct Backlight {
    b_min: u16,
    b_max: u16,
}

impl Backlight {
    pub fn init() -> Backlight {
        unsafe { backlight_init(); }
        let b_max = unsafe { backlight_max() } as u16;
        let b_min = unsafe { backlight_min() } as u16;
        Backlight {
            b_min, b_max
        }
    }

    pub fn get(&self) -> u8 {
        let b_cur = unsafe { backlight_get() };
        ((b_cur as f32 - self.b_min as f32) * 100.0 / (self.b_max - self.b_min) as f32).round() as u8
    }

    pub fn set(&self, value: u8) {
        let set_value = ((value as f32 / 100.0) * (self.b_max - self.b_min) as f32) as i32;
        unsafe { backlight_set(set_value); }
    }
}