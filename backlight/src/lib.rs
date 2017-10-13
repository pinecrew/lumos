extern { 
    fn backlight_init() -> i32;
    fn backlight_get() -> i32; 
    fn backlight_set(value: i32);
    fn backlight_max() -> i32;
    fn backlight_min() -> i32;
}

#[derive(Debug)]
pub struct Backlight {
    min: i32,
    max: i32,
}

impl Backlight {
    pub fn new() -> Backlight {
        let status = unsafe { backlight_init() };
        if status < 0 {
            panic!("something went wrong");
        }
        let max = unsafe { backlight_max() } as i32;
        let min = unsafe { backlight_min() } as i32;
        Backlight {
            min, max
        }
    }
    /// Get current backlight. Returns value in range [0, 1]
    pub fn get(&self) -> f32 {
        let b_cur = unsafe { backlight_get() };
        (b_cur - self.min) as f32 / (self.max - self.min) as f32
    }

    /// Set backlight. Value should be in range [0, 1]
    pub fn set(&self, value: f32) {
        let set_value = (value * (self.max - self.min) as f32) as i32;
        unsafe { backlight_set(set_value); }
    }
}