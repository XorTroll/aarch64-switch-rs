use enumflags2::BitFlags;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AbiConfigEntryKey {
    EndOfList = 0,
    MainThreadHandle = 1,
    NextLoadPath = 2,
    OverrideHeap = 3,
    OverrideService = 4,
    Argv = 5,
    SyscallAvailableHint = 6,
    AppletType = 7,
    AppletWorkaround = 8,
    Reserved9 = 9,
    ProcessHandle = 10,
    LastLoadResult = 11,
    RandomSeed = 14,
    UserIdStorage = 15,
    HosVersion = 16
}

#[derive(BitFlags, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AbiConfigEntryFlags {
    Mandatory = 0b1,
}

#[derive(BitFlags, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AbiConfigAppletFlags {
    ApplicationOverride = 0b1,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AbiConfigEntry {
    pub key: AbiConfigEntryKey,
    pub flags: BitFlags<AbiConfigEntryFlags>,
    pub value: [u64; 2],
}