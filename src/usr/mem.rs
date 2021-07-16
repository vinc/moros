use crate::{sys, usr, print};
use crate::sys::console::Style;
use alloc::string::ToString;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let width = sys::allocator::size().to_string().len();
    let color = Style::color("LightCyan");
    let reset = Style::reset();
    print!("{}Size:{} {:width$}\n", color, reset, sys::allocator::size(), width = width);
    print!("{}Used:{} {:width$}\n", color, reset, sys::allocator::used(), width = width);
    print!("{}Free:{} {:width$}\n", color, reset, sys::allocator::free(), width = width);
    usr::shell::ExitCode::CommandSuccessful
}
