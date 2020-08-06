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
/*
use nx::service;
use nx::service::fspsrv;
use nx::service::fspsrv::IFileSystemProxy;
use nx::service::fspsrv::IFileSystem;
*/
use nx::service::vi;
use nx::service::nv;
use nx::gpu;

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

pub fn gpu_test() -> Result<()> {
    let mut gpu_ctx: gpu::GpuContext<vi::SystemRootService, nv::AppletNvDrvService> = gpu::GpuContext::new(0x800000)?;

    let width: u32 = 1280;
    let height: u32 = 720;
    let color_fmt = gpu::ColorFormat::A8B8G8R8;
    let mut surface = gpu_ctx.create_stray_layer_surface("Default", width, height, 2, color_fmt, gpu::PixelFormat::RGBA_8888, gpu::Layout::BlockLinear)?;

    let mut x_pos: i32 = 0;
    let y_pos: i32 = 10;
    let x_incr: i32 = 5;
    let sq_length: i32 = 50;
    loop {
        let (buf, buf_size, slot, _has_fences, fences) = surface.dequeue_buffer(false)?;
        let mut surface_buf = surface_buffer::SurfaceBuffer::from(buf, buf_size, width, height, color_fmt);

        let c_white = 0xFFFFFFFF;
        let c_blue = 0xFF0000FF;
        surface_buf.clear(c_white);
        surface_buf.blit_with_color(x_pos, y_pos, sq_length, sq_length, c_blue);

        x_pos += x_incr;
        if (x_pos + sq_length) as u32 >= width {
            x_pos = 0;
        }

        surface.queue_buffer(slot, fences)?;
    }

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    diag_log!(log::LmLogger { log::LogSeverity::Info, false } => "Hello from {} and {} logging!", "Rust", "lm");

    if let Err(rc) = gpu_test() {
        assert::assert(assert::AssertMode::FatalThrow, rc);
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, ResultCode::from::<assert::ResultAssertionFailed>())
}