extern crate alloc;
use alloc::rc;

use linked_list_allocator::LockedHeap;
use core::cell;

pub type SharedObject<T> = rc::Rc<cell::RefCell<T>>;

pub fn make_shared<T>(t: T) -> SharedObject<T> {
    SharedObject::new(cell::RefCell::new(t))
}

pub const PAGE_ALIGNMENT: usize = 0x1000;

// TODO: switch from the spin crate linked_list_allocator uses to our lock system
// TODO: allocator failures

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn initialize(heap_address: *mut u8, heap_size: usize) {
    unsafe {
        GLOBAL_ALLOCATOR.lock().init(heap_address as usize, heap_size);
    }
}

pub fn flush_data_cache(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_mem_flush_data_cache(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_mem_flush_data_cache(address, size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    todo!();
}