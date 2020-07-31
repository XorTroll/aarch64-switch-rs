#[derive(Clone, Copy)]
#[repr(C)]
pub struct PointerAndSize {
    pub address: *mut u8,
    pub size: usize
}

impl PointerAndSize {
    pub const fn new(address: *mut u8, size: usize) -> Self {
        Self { address: address, size: size }
    }

    pub fn is_valid(&self) -> bool {
        !self.address.is_null() && (self.size != 0)
    }
}

#[macro_export]
macro_rules! bit {
    ($val:expr) => {
        (1 << $val)
    };
}

#[macro_export]
macro_rules! enum_define {
    ($name:ident($base_type:ident) { $( $field:ident = $value:expr ),* }) => {
        #[repr($base_type)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        pub enum $name {
            $( $field = $value, )*
        }
        
        impl $name {
            pub fn from(val: $base_type) -> Self {
                // TODO: better way to handle primitive type -> enum?
                unsafe {
                    core::mem::transmute(val)
                } 
            }
        }
        
        impl core::ops::BitAnd for $name {
            type Output = Self;
        
            fn bitand(self, rhs: Self) -> Self {
                Self::from(self as $base_type & rhs as $base_type)
            }
        }
        
        impl core::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }
        
        impl core::ops::BitOr for $name {
            type Output = Self;
        
            fn bitor(self, rhs: Self) -> Self {
                Self::from(self as $base_type | rhs as $base_type)
            }
        }
        
        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }
    };
}

#[macro_export]
macro_rules! util_return_if {
    ($cond:expr, $ret:expr) => {
        if $cond {
            return $ret;
        }
    }
}

#[macro_export]
macro_rules! util_return_unless {
    ($cond:expr, $ret:expr) => {
        if !$cond {
            return $ret;
        }
    }
}

use core::str;
use core::ptr;
use crate::result::*;

pub fn get_str_from_pointer(ptr: *mut u8, ptr_size: usize) -> Result<&'static str> {
    if ptr_size == 0 {
        return Err(ResultCode::new(0xBEEF1));
    }
    if ptr.is_null() {
        return Err(ResultCode::new(0xBEEF2));
    }
    unsafe {
        match core::str::from_utf8(core::slice::from_raw_parts_mut(ptr, ptr_size)) {
            Ok(name) => Ok(name.trim_matches('\0')),
            Err(_) => Err(ResultCode::new(0xBEEF3))
        }
    }
}

pub fn copy_str_to_pointer(string: &str, ptr: *mut u8) -> Result<()> {
    if string.is_empty() {
        return Err(ResultCode::new(0xBEEF1));
    }
    if ptr.is_null() {
        return Err(ResultCode::new(0xBEEF2));
    }

    unsafe {
        ptr::copy(string.as_ptr(), ptr, string.len());
    }
    Ok(())
}

macro_rules! get_mask_impl {
    ($start:expr, $end:expr) => {
        (bit!($end - $start + 1) - 1) << $start
    };
}

#[macro_export]
macro_rules! write_bits {
    ($start:expr, $end:expr, $value:expr, $data:expr) => {
        $value = ($value & (!get_mask_impl!($start, $end))) | ($data << $start);
    };
}

#[macro_export]
macro_rules! read_bits {
    ($start:expr, $end:expr, $value:expr) => {
        ($value & get_mask_impl!($start, $end)) >> $start
    };
}