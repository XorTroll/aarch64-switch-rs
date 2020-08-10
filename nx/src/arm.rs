global_asm!(include_str!("arm.s"));

pub fn cache_flush(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_arm_cache_flush(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_arm_cache_flush(address, size);
    }
}

pub fn get_system_tick() -> u64 {
    unsafe {
        let tick: u64;
        llvm_asm!("mrs x0, cntpct_el0" : "={x0}"(tick) ::: "volatile");

        tick
    }
}

pub fn get_system_tick_frequency() -> u64 {
    unsafe {
        let tick_freq: u64;
        llvm_asm!("mrs x0, cntfrq_el0" : "={x0}"(tick_freq) ::: "volatile");

        tick_freq
    }
}