use crate::{print, kernel};

pub fn login() {
    print!("\nUsername: ");
    let username = kernel::console::get_line();
    if username != "root\n" {
        kernel::sleep::sleep(1.0);
        login();
        return;
    }

    print!("Password: ");
    kernel::console::disable_echo();
    let password = kernel::console::get_line();
    kernel::console::enable_echo();
    print!("\n");
    if password != "root\n" {
        kernel::sleep::sleep(1.0);
        login();
    }
}
