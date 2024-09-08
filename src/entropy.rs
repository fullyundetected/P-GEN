use rand::{RngCore, rngs::OsRng};
use windows_sys::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};
use std::{ptr::addr_of_mut, sync::{Arc, Mutex}, thread::sleep, time::{Duration, SystemTime}};

use crate::crypto;

pub fn generate_bytes(length: u32) -> Vec<u8> {
    let mut data = vec![0; length as usize];
    OsRng.fill_bytes(&mut data);
    return data;
}

pub fn get_current_time_ns() -> u128 {
    let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    duration_since_epoch.as_nanos()
}

pub fn get_cursor_position() -> (u16, u16) {
    let mut point = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(addr_of_mut!(point));
    }
    ((point.x % 0x10000) as u16, (point.y % 0x10000) as u16)
}

pub fn mouse_position_entropy_updater_thread(mouse_position_entropy: Arc<Mutex<Vec<u8>>>) {
    let sleep_duration = Duration::from_millis(5);
    let mut i = 0;
    let mut last_mouse_position = (0, 0);
    loop {
        let current_mouse_position = get_cursor_position();
        
        if current_mouse_position.0 != last_mouse_position.0 || current_mouse_position.1 != last_mouse_position.1 {
            last_mouse_position = current_mouse_position;
            i = (i + 4) % 192;
            
            if let Ok(mut mouse_position_entropy_mutex_guard) = mouse_position_entropy.lock() {
                mouse_position_entropy_mutex_guard[i + 64] = (current_mouse_position.0 >> 8) as u8;
                mouse_position_entropy_mutex_guard[i + 65] = (current_mouse_position.0 & 0xff) as u8;
                mouse_position_entropy_mutex_guard[i + 66] = (current_mouse_position.1 >> 8) as u8;
                mouse_position_entropy_mutex_guard[i + 67] = (current_mouse_position.1 & 0xff) as u8;

                if i == 188 {
                    let entropy_hash = crypto::hashes::whirlpool_512_compute(&mouse_position_entropy_mutex_guard);
                    mouse_position_entropy_mutex_guard[0..64].clone_from_slice(&entropy_hash);
                }
            } else {
                mouse_position_entropy.clear_poison();
            }
        }
        
        sleep(sleep_duration);
    }
}