use crate::{print, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    let n = args.len();
    for i in 1..n {
        print!("{}", args[i]);
        if i < n {
            print!(" ");
        }
    }
    print!("\n");
    user::shell::ExitCode::CommandSuccessful
}
