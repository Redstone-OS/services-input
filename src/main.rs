#![no_std]
#![no_main]

extern crate redpowder;

use redpowder::input::{poll_keyboard, poll_mouse, KeyEvent};
use redpowder::prelude::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("(Service) Input Service Started");

    let mut key_buffer = [KeyEvent::default(); 32];

    loop {
        // Poll Keyboard
        match poll_keyboard(&mut key_buffer) {
            Ok(count) => {
                for i in 0..count {
                    let ev = key_buffer[i];
                    println!(
                        "(Input) Key: Scancode={:02X} Pressed={}",
                        ev.scancode, ev.pressed
                    );
                }
            }
            Err(_) => {} // Ignore errors
        }

        // Poll Mouse
        match poll_mouse() {
            Ok(mouse) => {
                if mouse.delta_x != 0 || mouse.delta_y != 0 || mouse.buttons != 0 {
                    println!(
                        "(Input) Mouse: X={} Y={} Buttons={:b} dX={} dY={}",
                        mouse.x, mouse.y, mouse.buttons, mouse.delta_x, mouse.delta_y
                    );
                }
            }
            Err(_) => {}
        }

        // Sleep to prevent 100% CPU usage
        sleep(10);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("(Service) Input Service Panic!");
    loop {}
}
