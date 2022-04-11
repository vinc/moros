use crate::usr;
use crate::api::syscall;
use crate::api::console::Style;
// use x86_64::addr::PhysAddr;
// use x86_64::structures::paging::frame::PhysFrame;
// use x86_64::registers::control::Cr3Flags;
// use x86_64::registers::control::Cr3;
use core::arch::asm;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}MOROS has reached its fate, the system is now rebooting.{}", csi_color, csi_reset);
    unsafe {
        // let addr = PhysAddr::new(0);
        // let frame = PhysFrame::containing_address(addr);
        // let flags = Cr3Flags::empty();
        // Cr3::write(frame, flags);
        asm!(
            "xor rax, rax",
            "mov cr3, rax"
        );
    }
    loop { syscall::sleep(1.0) }
}
