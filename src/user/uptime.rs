use crate::print;
use crate::kernel::clock;

pub fn main(args: &[&str]) {
    if args.len() == 2 && args[1] == "--metric" {
        let uptime = clock::uptime();
        if uptime < 1000.0 {
            print!("{:.2} seconds\n", uptime);
        } else {
            print!("{:.2} kiloseconds\n", uptime / 1000.0);
        }
    } else {
        let uptime = 0.0864 * clock::uptime();
        if uptime < 100.0 {
            print!("{:.2} dimidays\n", uptime);
        } else if uptime < 10_000.0 {
            print!("{:.2} centidays\n", uptime / 100.0);
        } else {
            print!("{:.2} days\n", uptime / 10_000.0);
        }
    }
}
