#![no_std]
#![no_main]

extern crate alloc;
extern crate redpowder;

use redpowder::input::{poll_keyboard, poll_mouse, KeyCode, KeyEvent, MouseState};
use redpowder::prelude::*;
use redpowder::window::opcodes;

/// Global allocator usando syscalls do kernel
#[global_allocator]
static ALLOCATOR: redpowder::mem::heap::SyscallAllocator = redpowder::mem::heap::SyscallAllocator;

/// Mensagem IPC para atualizar o compositor com novo estado de input
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct InputUpdateRequest {
    pub op: u32,          // opcodes::INPUT_UPDATE
    pub event_type: u32,  // 1=Key, 2=Mouse
    pub key_code: u32,    // KeyCode value
    pub key_pressed: u32, // 0=Released, 1=Pressed
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_buttons: u32,
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("");
    println!("==================================================");
    println!("   RedstoneOS Input Service v1.1.0");
    println!("==================================================");
    println!("(Input) Iniciando serviço...");

    // Conectar ao Compositor
    let compositor_port = loop {
        match redpowder::ipc::Port::connect("firefly.compositor") {
            Ok(p) => {
                println!("(Input) Conectado ao Compositor!");
                break p;
            }
            Err(_) => {
                println!("(Input) Aguardando Compositor...");
                sleep(100);
            }
        }
    };

    let mut key_buffer = [KeyEvent::default(); 32];
    let mut last_mouse = MouseState::default();

    let mut loop_count: u64 = 0;
    loop {
        loop_count += 1;
        if loop_count % 100 == 0 {
            // println!("(Input) Loop ativo..."); // TODO: Remover futuramente
        }

        // 1. Processar Teclado
        if let Ok(count) = poll_keyboard(&mut key_buffer) {
            for i in 0..count {
                let ev = key_buffer[i];
                let key_code = KeyCode::from_scancode(ev.scancode);

                if key_code != KeyCode::None {
                    let req = InputUpdateRequest {
                        op: opcodes::INPUT_UPDATE,
                        event_type: 1, // Key
                        key_code: key_code as u32,
                        key_pressed: if ev.pressed { 1 } else { 0 },
                        mouse_x: 0,
                        mouse_y: 0,
                        mouse_buttons: 0,
                    };

                    let bytes = unsafe {
                        core::slice::from_raw_parts(
                            &req as *const _ as *const u8,
                            core::mem::size_of::<InputUpdateRequest>(),
                        )
                    };

                    // TODO: Remover logs de debug após teste
                    println!(
                        "(Input) Sending Key: {:?} (pressed={})",
                        key_code, ev.pressed
                    );
                    let _ = compositor_port.send(bytes, 0);
                }
            }
        }

        // 2. Processar Mouse
        if let Ok(mouse) = poll_mouse() {
            if mouse.x != last_mouse.x
                || mouse.y != last_mouse.y
                || mouse.buttons != last_mouse.buttons
            {
                // TODO: Remover log de mouse futuramente
                println!(
                    "(Input) Mouse: ({}, {}) buttons={}",
                    mouse.x, mouse.y, mouse.buttons
                );

                let req = InputUpdateRequest {
                    op: opcodes::INPUT_UPDATE,
                    event_type: 2, // Mouse
                    key_code: 0,
                    key_pressed: 0,
                    mouse_x: mouse.x,
                    mouse_y: mouse.y,
                    mouse_buttons: mouse.buttons as u32,
                };

                let bytes = unsafe {
                    core::slice::from_raw_parts(
                        &req as *const _ as *const u8,
                        core::mem::size_of::<InputUpdateRequest>(),
                    )
                };

                let _ = compositor_port.send(bytes, 0);
                last_mouse = mouse;
            }
        }

        // Sleep curto para não saturar a CPU nem o IPC
        sleep(16); // ~60Hz
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("(Input) PANIC: {:?}", info);
    loop {
        sleep(1000);
    }
}
