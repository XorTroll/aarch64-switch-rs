#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::diag::log::Logger;
use nx::service::vi;
use nx::service::nv;
use nx::gpu;
use nx::service::hid;
use nx::input;

use core::panic;

mod surface_buffer;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

pub fn gpu_main() -> Result<()> {
    let mut gpu_ctx: gpu::GpuContext<vi::SystemRootService, nv::AppletNvDrvService> = gpu::GpuContext::new(0x800000)?;
    let mut input_ctx = input::InputContext::new(0, hid::NpadStyleTag::ProController | hid::NpadStyleTag::Handheld | hid::NpadStyleTag::JoyconPair | hid::NpadStyleTag::JoyconLeft | hid::NpadStyleTag::JoyconRight | hid::NpadStyleTag::SystemExt | hid::NpadStyleTag::System, vec![hid::ControllerId::Player1, hid::ControllerId::Player2, hid::ControllerId::Player3, hid::ControllerId::Player4, hid::ControllerId::Player5, hid::ControllerId::Player6, hid::ControllerId::Player7, hid::ControllerId::Player8, hid::ControllerId::Handheld])?;

    let width: u32 = 1280;
    let height: u32 = 720;
    let color_fmt = gpu::ColorFormat::A8B8G8R8;
    let mut surface = gpu_ctx.create_stray_layer_surface("Default", width, height, 2, color_fmt, gpu::PixelFormat::RGBA_8888, gpu::Layout::BlockLinear)?;

    let mut x_pos: i32 = 50;
    let mut y_pos: i32 = 50;
    let mut x_incr: i32 = 1;
    let mut y_incr: i32 = 1;
    let mut x_mult: i32 = 4;
    let mut y_mult: i32 = 4;
    let shape_size: i32 = 50;
    let c_white: u32 = 0xFFFFFFFF;
    let c_blue: u32 = 0xFFFF0000;

    loop {
        let mut input_player = match input_ctx.is_controller_connected(hid::ControllerId::Player1) {
            true => input_ctx.get_player(hid::ControllerId::Player1),
            false => input_ctx.get_player(hid::ControllerId::Handheld)
        }?;
        let input_keys = input_player.get_button_state_down();
        if input_keys.contains(input::Key::Plus) {
            diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Plus pressed -> exiting...");
            // Exit if Plus/+ is pressed.
            break;
        }

        let (buf, buf_size, slot, has_fences, fences) = surface.dequeue_buffer(true)?;
        let mut surface_buf = surface_buffer::SurfaceBuffer::from(buf, buf_size, width, height, color_fmt);
        
        surface_buf.clear(c_white);
        surface_buf.blit_with_color(x_pos, y_pos, shape_size, shape_size, c_blue);
        surface_buf.draw_text(format!("Hello world from aarch64-switch-rs!\nPress + to exit this demo.\n\nBox position: ({}, {})", x_pos, y_pos), 2, c_blue, 10, 10);

        x_pos += x_incr * x_mult;
        y_pos += y_incr * y_mult;

        if x_pos <= 0 {
            if x_incr < 0 {
                x_incr = -x_incr;
            }
            x_pos += x_incr * x_mult;
            x_mult += 1;
        }
        else if (x_pos + shape_size) as u32 >= width {
            if x_incr > 0 {
                x_incr = -x_incr;
            }
            x_pos += x_incr * x_mult;
            x_mult += 1;
        }

        if y_pos <= 0 {
            if y_incr < 0 {
                y_incr = -y_incr;
            }
            y_pos += y_incr * y_mult;
            y_mult += 1;
        }
        else if (y_pos + shape_size) as u32 >= height {
            if y_incr > 0 {
                y_incr = -y_incr;
            }
            y_pos += y_incr * y_mult;
            y_mult += 1;
        }

        surface.queue_buffer(slot, fences)?;
    }

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    if let Err(rc) = gpu_main() {
        assert::assert(assert::AssertMode::FatalThrow, rc);
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, ResultCode::from::<assert::ResultAssertionFailed>())
}