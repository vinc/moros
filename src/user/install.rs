use crate::{print, kernel, user};
use crate::kernel::vga::Color;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let (fg, bg) = kernel::vga::color();
    kernel::vga::set_color(Color::LightCyan, bg);
    print!("Welcome to MOROS v{} installation program!\n", env!("CARGO_PKG_VERSION"));
    kernel::vga::set_color(fg, bg);
    print!("\n");

    print!("Proceed? [y/N] ");
    if kernel::console::get_line().trim() == "y" {
        print!("\n");

        if !kernel::fs::is_mounted() {
            kernel::vga::set_color(Color::LightCyan, bg);
            print!("Listing disks ...\n");
            kernel::vga::set_color(fg, bg);
            user::disk::main(&["disk", "list"]);
            print!("\n");

            kernel::vga::set_color(Color::LightCyan, bg);
            print!("Formatting disk ...\n");
            kernel::vga::set_color(fg, bg);
            print!("Enter path of disk to format: ");
            let pathname = kernel::console::get_line();
            let res = user::disk::main(&["disk", "format", pathname.trim_end()]);
            if res == user::shell::ExitCode::CommandError {
                return res;
            }
            print!("\n");
        }

        kernel::vga::set_color(Color::LightCyan, bg);
        print!("Populating filesystem ...\n");
        kernel::vga::set_color(fg, bg);
        create_dir("/bin"); // Binaries
        create_dir("/dev"); // Devices
        create_dir("/ini"); // Initializers
        create_dir("/lib"); // Libraries
        create_dir("/net"); // Network
        create_dir("/src"); // Sources
        create_dir("/tmp"); // Temporaries
        create_dir("/usr"); // User directories
        create_dir("/var"); // Variables

        copy_file("/ini/boot.sh", include_str!("../../dsk/ini/boot.sh"));
        copy_file("/ini/banner.txt", include_str!("../../dsk/ini/banner.txt"));
        copy_file("/tmp/alice.txt", include_str!("../../dsk/tmp/alice.txt"));

        if kernel::process::user().is_none() {
            print!("\n");
            kernel::vga::set_color(Color::LightCyan, bg);
            print!("Creating user ...\n");
            kernel::vga::set_color(fg, bg);
            let res = user::user::main(&["user", "create"]);
            if res == user::shell::ExitCode::CommandError {
                return res;
            }
        }

        print!("\n");
        kernel::vga::set_color(Color::LightCyan, bg);
        print!("Installation successful!\n");
        kernel::vga::set_color(fg, bg);
        print!("\n");
        print!("Exit console or reboot to apply changes\n");
    }

    user::shell::ExitCode::CommandSuccessful
}

fn create_dir(pathname: &str) {
    if kernel::fs::Dir::create(pathname).is_some() {
        print!("Created '{}'\n", pathname);
    }
}

fn copy_file(pathname: &str, contents: &str) {
    if kernel::fs::File::open(pathname).is_some() {
        return;
    }
    if let Some(mut file) = kernel::fs::File::create(pathname) {
        file.write(&contents.as_bytes()).unwrap();
        print!("Copied '{}'\n", pathname);
    }
}
