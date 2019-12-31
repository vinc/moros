use crate::{print, kernel};

pub fn login() {
    print!("Username: ");
    let username = kernel::console::get_line();
    if username != "root\n" {
        kernel::sleep::sleep(1.0);
        login();
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
