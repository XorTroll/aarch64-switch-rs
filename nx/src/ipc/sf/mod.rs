use super::*;
use core::mem;

#[derive(Copy, Clone)]
pub struct Buffer<const A: BufferAttribute> {
    pub buf: *const u8,
    pub size: usize
}

impl<const A: BufferAttribute> Buffer<A> {
    pub const fn from_const<T>(buf: *const T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_mut<T>(buf: *mut T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_var<T>(var: &T) -> Self {
        Self::from_const(var as *const T, mem::size_of::<T>())
    }

    pub const fn from_array<T>(arr: &[T]) -> Self {
        Self::from_const(arr.as_ptr(), arr.len() * mem::size_of::<T>())
    }

    pub const fn get_as<T>(&self) -> &T {
        unsafe {
            &*(self.buf as *const T)
        }
    }
}

pub type InMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [In, MapAlias] }}>;
pub type OutMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [Out, MapAlias] }}>;
pub type InAutoSelectBuffer = Buffer<{bit_group!{ BufferAttribute [In, AutoSelect] }}>;
pub type OutAutoSelectBuffer = Buffer<{bit_group!{ BufferAttribute [Out, AutoSelect] }}>;
pub type InPointerBuffer = Buffer<{bit_group!{ BufferAttribute [In, Pointer] }}>;
pub type OutPointerBuffer = Buffer<{bit_group!{ BufferAttribute [Out, Pointer] }}>;

#[derive(Copy, Clone)]
pub struct Handle<const M: HandleMode> {
    pub handle: svc::Handle
}

impl<const M: HandleMode> Handle<M> {
    pub const fn from(handle: svc::Handle) -> Self {
        Self { handle: handle }
    }
}

pub type CopyHandle = Handle<{HandleMode::Copy}>;
pub type MoveHandle = Handle<{HandleMode::Move}>;

#[derive(Copy, Clone)]
pub struct ProcessId {
    pub process_id: u64
}

impl ProcessId {
    pub const fn from(process_id: u64) -> Self {
        Self { process_id: process_id }
    }

    pub const fn new() -> ProcessId {
        Self::from(0)
    }
}

pub mod sm;

pub mod psm;

pub mod applet;

pub mod lm;

pub mod fatal;

pub mod dispdrv;

pub mod fspsrv;

pub mod hid;

pub mod nv;

pub mod vi;