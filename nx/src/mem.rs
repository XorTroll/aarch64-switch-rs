extern crate alloc;

use crate::result::*;
use crate::diag::assert;
use linked_list_allocator::LockedHeap;
use alloc::rc;
use core::cell;

pub type SharedObject<T> = rc::Rc<cell::RefCell<T>>;

pub fn make_shared<T>(t: T) -> SharedObject<T> {
    SharedObject::new(cell::RefCell::new(t))
}

pub const PAGE_ALIGNMENT: usize = 0x1000;

pub const RESULT_SUBMODULE: u32 = 10;

result_lib_define_group!(RESULT_SUBMODULE => {
    ResultMemoryAllocationFailed: 1
});

// TODO: switch from the spin crate linked_list_allocator uses to our lock system

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn initialize(heap_address: *mut u8, heap_size: usize) {
    unsafe {
        GLOBAL_ALLOCATOR.lock().init(heap_address as usize, heap_size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    let rc: Result<()> = Err(ResultCode::from::<ResultMemoryAllocationFailed>());
    rc.unwrap();
    loop {}
}