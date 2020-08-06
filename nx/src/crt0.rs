global_asm!(include_str!("crt0.s"));

use crate::svc;
use crate::mem;
use crate::dynamic;
use crate::sync;
use crate::util;
use crate::hbl;
use crate::thread;
use crate::result::*;

use core::option;
use core::ptr;

extern "Rust" {
    fn main() -> Result<()>;
    fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize;
}

pub type ExitFn = fn(ResultCode);

static mut G_EXIT_FN: sync::Locked<option::Option<ExitFn>> = sync::Locked::new(false, None);
static mut G_MAIN_THREAD: thread::Thread = thread::Thread::new();

unsafe fn initialize_tls_main_thread_impl(thread_handle: svc::Handle) {
    G_MAIN_THREAD = thread::Thread::existing(thread_handle, "MainThread", ptr::null_mut(), 0, false).unwrap();
    let mut tls = thread::get_thread_local_storage();
    (*tls).thread_ref = &mut G_MAIN_THREAD;
}

#[no_mangle]
unsafe fn __nx_crt0_entry(abi_ptr: *const hbl::AbiConfigEntry, raw_main_thread_handle: u64, aslr_base_address: *const u8, lr_exit_fn: ExitFn, bss_start: *mut u8, bss_end: *mut u8) {
    let is_hbl_nro = !abi_ptr.is_null() && (raw_main_thread_handle == u64::MAX);
    
    // Clear .bss section
    let bss_size = bss_end as usize - bss_start as usize;
    ptr::write_bytes(bss_start, 0, bss_size);

    // Relocate ourselves
    dynamic::relocate(aslr_base_address).unwrap();

    let mut heap = util::PointerAndSize::new(ptr::null_mut(), 0);
    let mut main_thread_handle = raw_main_thread_handle as svc::Handle;

    // If homebrew NRO, parse the config entries hbloader sent us
    if is_hbl_nro {
        let mut abi_entry = abi_ptr;
        loop {
            match (*abi_entry).key {
                hbl::AbiConfigEntryKey::EndOfList => {
                    break;
                },
                hbl::AbiConfigEntryKey::OverrideHeap => {
                    heap.address = (*abi_entry).value[0] as *mut u8;
                    heap.size = (*abi_entry).value[1] as usize;
                },
                hbl::AbiConfigEntryKey::MainThreadHandle => {
                    main_thread_handle = (*abi_entry).value[0] as svc::Handle;
                }
                _ => {
                    
                }
            }
            abi_entry = abi_entry.offset(1);
        }
    }

    initialize_tls_main_thread_impl(main_thread_handle);

    // Set exit function (will be null for non-hbl NROs)
    if is_hbl_nro {
        G_EXIT_FN.set(Some(lr_exit_fn));
    }
    else {
        G_EXIT_FN.set(None);
    }
    
    // Initialize memory allocation
    heap = initialize_heap(heap);
    mem::initialize(heap.address, heap.size);

    // TODO: finish implementing CRT0

    // Call main() and get its result code
    let rc = match main() {
        Ok(()) => ResultCode::from::<ResultSuccess>(),
        Err(rc) => rc,
    };

    // Exit
    exit(rc);
}

#[no_mangle]
unsafe fn __nx_crt0_exception_entry(_error_desc: u32, _stack_top: *mut u8) {
    svc::return_from_exception(ResultCode::from::<svc::ResultUnhandledException>());
}

pub fn exit(rc: ResultCode) -> ! {
    unsafe {
        match G_EXIT_FN.get() {
            Some(exit_fn) => {
                exit_fn(rc);
            },
            None => {
                svc::exit_process();
            }
        }
    }
    loop {}
}