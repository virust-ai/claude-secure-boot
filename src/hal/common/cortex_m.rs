/// Cortex-M4 specific functionality

/// Enable interrupts
#[inline(always)]
pub fn enable_interrupts() {
    unsafe { core::arch::asm!("cpsie i") };
}

/// Disable interrupts
#[inline(always)]
pub fn disable_interrupts() {
    unsafe { core::arch::asm!("cpsid i") };
}

/// Execute WFI (Wait For Interrupt) instruction
#[inline(always)]
pub fn wait_for_interrupt() {
    unsafe { core::arch::asm!("wfi") };
}

/// Execute DSB (Data Synchronization Barrier) instruction
#[inline(always)]
pub fn data_synchronization_barrier() {
    unsafe { core::arch::asm!("dsb") };
}

/// Execute ISB (Instruction Synchronization Barrier) instruction
#[inline(always)]
pub fn instruction_synchronization_barrier() {
    unsafe { core::arch::asm!("isb") };
}

/// Breakpoint instruction
#[inline(always)]
pub fn breakpoint() {
    unsafe { core::arch::asm!("bkpt") };
}