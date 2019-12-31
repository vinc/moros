use crate::{print, kernel};

pub fn main(args: &[&str]) {
    if args.len() != 2 {
        return;
    }

    let pathname = args[1];
    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", pathname);
    } else if let Some(mut file) = kernel::fs::File::create(pathname) {
        file.write("fake contents");
    } else {
        print!("Could not write to '{}'\n", pathname);
    }
}
