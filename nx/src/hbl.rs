enum_define!(AbiConfigEntryKey(u32) {
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
});

enum_define!(AbiConfigEntryFlags(u32) {
    Mandatory = bit!(0)
});

enum_define!(AbiConfigAppletFlags(u32) {
    ApplicationOverride = bit!(0)
});

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AbiConfigEntry {
    pub key: AbiConfigEntryKey,
    pub flags: AbiConfigEntryFlags,
    pub value: [u64; 2],
}