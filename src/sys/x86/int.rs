use super::reg;

use core::arch::asm;

#[inline]
pub fn enable_interrupts() {
    // NOTE: interrupts are not enabled until after the next instruction
    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack, preserves_flags));
    }
}

#[inline]
pub fn without_interrupts<F, R>(f: F) -> R where F: FnOnce() -> R {
    let enabled = are_interrupts_enabled();
    if enabled {
        disable_interrupts();
    }
    let res = f();
    if enabled {
        enable_interrupts();
    }
    res
}

#[inline]
fn are_interrupts_enabled() -> bool {
    let flags: usize;
    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags));

        #[cfg(target_arch = "x86")]
        asm!("pushfd; pop {}", out(reg) flags, options(nomem, preserves_flags));
    }
    flags & reg::flags::IF != 0
}

#[test_case]
fn test_without_interrupts() {
    disable_interrupts();

    assert!(!are_interrupts_enabled());
    assert!(!without_interrupts(|| are_interrupts_enabled()));
    assert!(!are_interrupts_enabled());

    enable_interrupts();

    assert!(are_interrupts_enabled());
    assert!(!without_interrupts(|| are_interrupts_enabled()));
    assert!(are_interrupts_enabled());
}
