#![no_std]
#![no_main]

extern crate alloc;
extern crate redpowder;

use redpowder::input::{poll_keyboard, poll_mouse, KeyEvent};
use redpowder::prelude::*;

/// Global allocator usando syscalls do kernel
#[global_allocator]
static ALLOCATOR: redpowder::mem::heap::SyscallAllocator = redpowder::mem::heap::SyscallAllocator;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("(Service) Input Service Started");

    // Tenta conectar ao Shell (retry loop)
    let shell_port = loop {
        match redpowder::ipc::Port::connect("shell_input") {
            Ok(p) => {
                println!("(Service) Connected to Shell!");
                break p;
            }
            Err(_) => {
                // Shell ainda não criou a porta, esperar e tentar de novo
                println!("(Service) Waiting for Shell...");
                sleep(100);
            }
        }
    };

    let mut key_buffer = [KeyEvent::default(); 32];

    loop {
        // Poll Keyboard
        match poll_keyboard(&mut key_buffer) {
            Ok(count) => {
                for i in 0..count {
                    let ev = key_buffer[i];
                    // Serializa e envia
                    // Struct Layout: [scancode (1 byte), pressed (1 byte), padding (6 bytes)] -> 8 bytes?
                    // KeyEvent é struct { scancode: u8, pressed: bool } -> alignment padding.
                    // Vamos enviar manualmente como bytes
                    let mut packet = [0u8; 2];
                    packet[0] = ev.scancode;
                    packet[1] = if ev.pressed { 1 } else { 0 };

                    if let Err(_) = shell_port.send(&packet, 0) {
                        println!("(Input) Failed to send to shell");
                    }
                }
            }
            Err(_) => {} // Ignore errors
        }

        // Poll Mouse (ainda loga no console por enquanto)
        match poll_mouse() {
            Ok(mouse) => {
                if mouse.delta_x != 0 || mouse.delta_y != 0 || mouse.buttons != 0 {
                    // Futuro: enviar para Compositor
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
