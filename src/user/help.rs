use crate::{print, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    print!("c[opy] <file> <file>      Copy file from source to destination\n");
    print!("d[elete] <file>           Delete file or empty directory\n");
    print!("e[dit] <file>             Edit existing or new file\n");
    print!("h[elp]                    Display this text\n");
    print!("l[ist] <dir>              List entries in directory\n");
    print!("m[ove] <file> <file>      Move file from source to destination\n");
    print!("p[rint] <string>          Print string to screen\n");
    print!("q[uit]                    Quit the shell\n");
    print!("r[ead] <file>             Read file to screen\n");
    print!("w[rite] <file>            Write file or directory\n");
    user::shell::ExitCode::CommandSuccessful
}
