use crate::{print, kernel};

pub fn login() {
    print!("Username: ");
    let username = kernel::console::get_line();
    print!("\n");
    if username != "root\n" {
        kernel::sleep::sleep(1.0);
        login();
    }

    print!("Password: ");
    let password = kernel::console::get_line();
    print!("\n");
    if password != "root\n" {
        kernel::sleep::sleep(1.0);
        login();
    }
}
