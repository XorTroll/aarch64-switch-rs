#![no_std]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_raw_ptr_to_usize_cast)]
#![macro_use]

#[macro_use]
pub mod result;

#[macro_use]
pub mod util;

pub mod alloc;

pub mod dynamic;

pub mod sync;

pub mod thread;

pub mod hbl;

#[macro_use]
pub mod crt0;

pub mod svc;

pub mod ipc;

#[macro_use]
pub mod service;