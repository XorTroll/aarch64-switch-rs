#![no_std]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(const_raw_ptr_deref)]
#![macro_use]

#[macro_use]
extern crate alloc;

#[macro_use]
pub mod macros;

pub mod result;

pub mod util;

pub mod mem;

pub mod dynamic;

pub mod sync;

pub mod thread;

pub mod hbl;

pub mod crt0;

pub mod svc;

pub mod ipc;

pub mod service;

pub mod diag;

pub mod gpu;

pub mod input;

pub mod vmem;

pub mod arm;