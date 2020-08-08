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
use nx::service::hid;
use nx::input;

use core::panic;

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

pub fn input_test() -> Result<()> {
    let mut input_ctx = input::InputContext::new(0, hid::NpadStyleTag::ProController | hid::NpadStyleTag::Handheld | hid::NpadStyleTag::JoyconPair | hid::NpadStyleTag::JoyconLeft | hid::NpadStyleTag::JoyconRight | hid::NpadStyleTag::SystemExt | hid::NpadStyleTag::System, vec![hid::ControllerId::Player1, hid::ControllerId::Player2, hid::ControllerId::Player3, hid::ControllerId::Player4, hid::ControllerId::Player5, hid::ControllerId::Player6, hid::ControllerId::Player7, hid::ControllerId::Player8, hid::ControllerId::Handheld])?;

    loop {
        let mut input_player = match input_ctx.is_controller_connected(hid::ControllerId::Player1) {
            true => input_ctx.get_player(hid::ControllerId::Player1),
            false => input_ctx.get_player(hid::ControllerId::Handheld)
        }?;

        let input_keys = input_player.get_button_state_down();
        if input_keys.contains(input::Key::A) {
            diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "A was pressed by {:?}!", input_player.get_controller());
            break;
        }
    }

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    if let Err(rc) = input_test() {
        assert::assert(assert::AssertMode::FatalThrow, rc);
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, ResultCode::from::<assert::ResultAssertionFailed>())
}