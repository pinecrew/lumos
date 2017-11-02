extern crate glob;
extern crate libc;

use glob::glob;
use std::os::unix::io::AsRawFd;
use std::{mem, fs, io};
use std::io::Read;
use std::fs::File;

fn get_fd() -> Option<File> {
    let event_device_mask = "/dev/input/event*";
    let event_device_name = "Asus WMI hotkeys";
    match glob(event_device_mask) {
        Err(e) => panic!("Cannot glob({}): {}", event_device_mask, e),
        Ok(dir) => {
            for opt_item in dir {
                match opt_item {
                    Err(e) => panic!("Cannot get path from glob: {}", e),
                    Ok(item) => {
                        match fs::File::open(&item) {
                            Err(e) => panic!("Cannot open {}: {}", item.to_string_lossy(), e),
                            Ok(fd) => {
                                let mut buffer = [0u8; 256];
                                let rc = unsafe {
                                    libc::ioctl(fd.as_raw_fd(), 0x8_100_45_06, &mut buffer)
                                };
                                if rc == -1 {
                                    panic!("Cannot get device name for {}: errno {}",
                                           item.to_string_lossy(),
                                           io::Error::last_os_error());
                                } else {
                                    let name = String::from_utf8_lossy(&buffer[0..rc as usize]);
                                    if name.starts_with(event_device_name) {
                                        println!("fd = {:?}, name = {:?}", fd, name);
                                        return Some(fd);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn main() {
    let mut fd = get_fd().unwrap();
    let rc = unsafe {
        use std::ptr::null_mut;

        let mut set: libc::fd_set = mem::zeroed();
        let mut timeval: libc::timeval = mem::zeroed();
        timeval.tv_sec = 1 as i64;
        libc::FD_SET(fd.as_raw_fd(), &mut set);
        libc::select(fd.as_raw_fd() + 1,
                     &mut set,
                     null_mut(),
                     null_mut(),
                     &mut timeval)
    };

    if rc == -1 {
        panic!("Cannot select on event fd: {}", io::Error::last_os_error());
    }

    #[repr(C)]
    #[derive(Debug)]
    struct Event {
        sec: i64,
        usec: i64,
        event_type: u16,
        code: u16,
        value: i32,
    }

    loop {
        const SIZE: usize = 24;
        assert_eq!(SIZE, mem::size_of::<Event>());
        let event = unsafe {
            let mut event: Event = mem::zeroed();
            let mut u: &mut [u8; SIZE] = mem::transmute(&mut event);
            if let Err(e) = fd.read_exact(u) {
                panic!("Cannot read from event device: {}", e);
            }
            event
        };
        let result = event.event_type == 1 /*KEY*/ && event.code == 0x230/*KEY_ALS_TOGGLE*/ && event.value == 1;
        println!("result = {}, {}", result, event.sec);
    }
}
