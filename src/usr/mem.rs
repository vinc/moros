use crate::{sys, usr};
use crate::api::console::Style;
use alloc::string::ToString;

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    let width = sys::allocator::size().to_string().len();
    let color = Style::color("LightCyan");
    let reset = Style::reset();
    println!("{}Size:{} {:width$}", color, reset, sys::allocator::size(), width = width);
    println!("{}Used:{} {:width$}", color, reset, sys::allocator::used(), width = width);
    println!("{}Free:{} {:width$}", color, reset, sys::allocator::free(), width = width);
    usr::shell::ExitCode::CommandSuccessful
}
