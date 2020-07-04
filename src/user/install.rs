use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    print!("Welcome to MOROS v{} installation program!\n", env!("CARGO_PKG_VERSION"));
    print!("Proceed? [y/N] ");
    if kernel::console::get_line().trim() == "y" {
        if !kernel::fs::is_mounted() {
            print!("MFS is not mounted to '/'\n");
            return user::shell::ExitCode::CommandError;
        }

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
