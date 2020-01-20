use crate::{print, kernel, user};
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let mut pathname = args[1];

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = kernel::fs::Dir::open(pathname) {
        //let mut files: Vec<_> = dir.read().collect();
        let mut files: Vec<_> = dir.read().collect();

        // With `std::Vec` we would have used the following to sort the files:
        //
        // files.sort_by_key(|f| f.name());
        let n = files.len();
        for i in 0..n {
            let mut min = i;
            for j in (i + 1)..n {
                if files[j].name().as_str() < files[min].name().as_str() {
                    min = j
                }
            }
            if min != i {
                // The implementation of `heapless::String` doesn't have Copy,
                // so `DirEntry` that use it doesn't either, therefor the
                // following is not possible:
                //
                // let tmp = files[i];
                // files[i] = files[min];
                // files[min] = tmp;
                let tmp = files.swap_remove(min);
                //files.push(tmp);
                files.push(tmp);
                let tmp = files.swap_remove(i);
                //files.push(tmp);
                files.push(tmp);
            }
        }

        for file in files {
            print!("{}\n", file.name());
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Dir not found '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}
