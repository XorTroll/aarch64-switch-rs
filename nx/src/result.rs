use core::result;
use core::fmt;

const MODULE_BITS: u32 = 9;
const DESCRIPTION_BITS: u32 = 13;
const DEFAULT_VALUE: u32 = 0;
const SUCCESS_VALUE: u32 = DEFAULT_VALUE;

pub const fn pack_value(module: u32, description: u32) -> u32 {
    module | (description << MODULE_BITS)
}

pub const fn unpack_module(value: u32) -> u32 {
    value & !(!DEFAULT_VALUE << MODULE_BITS)
}

pub const fn unpack_description(value: u32) -> u32 {
    (value >> MODULE_BITS) & !(!DEFAULT_VALUE << DESCRIPTION_BITS)
}

pub trait ResultBase {
    fn get_value() -> u32;
    fn get_module() -> u32;
    fn get_description() -> u32;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ResultCode {
    value: u32
}

impl ResultCode {
    pub const fn new(value: u32) -> Self {
        Self { value: value }
    }
    
    pub fn from<T: ResultBase>() -> Self {
        Self { value: T::get_value() }
    }
    
    pub const fn is_success(&self) -> bool {
        self.value == SUCCESS_VALUE
    }
    
    pub const fn is_failure(&self) -> bool {
        !self.is_success()
    }
    
    pub const fn get_value(&self) -> u32 {
        self.value
    }
    
    pub const fn get_module(&self) -> u32 {
        unpack_module(self.value)
    }
    
    pub const fn get_description(&self) -> u32 {
        unpack_description(self.value)
    }
    
    pub fn matches<T: ResultBase>(&self) -> bool {
        T::get_value() == self.value
    }
}

impl fmt::Debug for ResultCode {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{:#X}", self.value)
    }
}

impl fmt::Display for ResultCode {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{:0>4}-{:0>4}", 2000 + self.get_module(), self.get_description())
    }
}

pub const LIBRARY_MODULE: u32 = 430;

result_define!(ResultSuccess: 0);

pub type Result<T> = result::Result<T, ResultCode>;

pub fn wrap<T>(rc: ResultCode, value: T) -> Result<T> {
    if rc.is_success() {
        Ok(value)
    }
    else {
        Err(rc)
    }
}