use crate::result::*;

enum_define!(Tag(i64) {
    Invalid = 0,
    Needed = 1,
    PltRelSize = 2,
    Hash = 4,
    StrTab = 5,
    SymTab = 6,
    RelaOffset = 7,
    RelaSize = 8,
    RelaEntrySize = 9,
    SymEnt = 11,
    RelOffset = 17,
    RelSize = 18,
    RelEntrySize = 19,
    PltRel = 20,
    JmpRel = 23,
    InitArray = 25,
    FiniArray = 26,
    InitArraySize = 27,
    FiniArraySize = 28,
    RelaCount = 0x6FFFFFF9
});

enum_define!(RelocationType(u32) {
    AArch64Abs64 = 257,
    AArch64GlobDat = 1025,
    AArch64JumpSlot = 1026,
    AArch64Relative = 1027
});

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dyn {
    pub tag: Tag,
    pub val_ptr: u64,
}

use core::ptr;

impl Dyn {
    pub unsafe fn find_value(&self, tag: Tag) -> Result<u64> {
        let mut found: *const u64 = ptr::null();
        let mut self_ptr = self as *const Self;
        while (*self_ptr).tag != Tag::Invalid {
            if (*self_ptr).tag == tag {
                if found.is_null() {
                    found = &(*self_ptr).val_ptr;
                }
                else {
                    return Err(ResultCode::new(0xDEAD));
                }
            }
            self_ptr = self_ptr.offset(1);
        }
        if found.is_null() {
            return Err(ResultCode::new(0xDEAD));
        }
        Ok(*found)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InfoSymbol {
    pub relocation_type: RelocationType,
    pub symbol: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Info {
    pub value: u64,
    pub symbol: InfoSymbol,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rela {
    pub offset: u64,
    pub info: Info,
    pub addend: i64,
}