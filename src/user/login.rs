use crate::{print, kernel, user};

// TODO: Add max number of attempts
pub fn login() -> user::shell::ExitCode {
    print!("\nUsername: ");
    let username = kernel::console::get_line();
    if username != "root\n" {
        kernel::sleep::sleep(1.0);
        return login();
    }

    print!("Password: ");
    kernel::console::disable_echo();
    let password = kernel::console::get_line();
    kernel::console::enable_echo();
    print!("\n");
    if password != "root\n" {
        kernel::sleep::sleep(1.0);
        return login();
    }
    user::shell::ExitCode::CommandSuccessful
}
