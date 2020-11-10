use crate::{kernel, print, user};
use crate::kernel::console::Style;
use alloc::string::ToString;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let width = kernel::allocator::size().to_string().len();
    let color = Style::color("LightCyan");
    let reset = Style::reset();
    print!("{}Size:{} {:width$}\n", color, reset, kernel::allocator::size(), width = width);
    print!("{}Used:{} {:width$}\n", color, reset, kernel::allocator::used(), width = width);
    print!("{}Free:{} {:width$}\n", color, reset, kernel::allocator::free(), width = width);
    user::shell::ExitCode::CommandSuccessful
}
