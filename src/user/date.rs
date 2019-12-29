use crate::print;
use crate::kernel::clock;

pub fn main(args: &[&str]) {
    let time = clock::clock_realtime();

    if args.len() == 2 && args[1] == "--metric" {
        if time < 1000.0 {
            print!("{:.2} seconds\n", time);
        } else {
            print!("{:.2} kiloseconds\n", time / 1000.0);
        }
    } else {
        let time = 0.0864 * time;
        if time < 100.0 {
            print!("{:.2} dimidays\n", time);
        } else if time < 10_000.0 {
            print!("{:.2} centidays\n", time / 100.0);
        } else {
            print!("{:.2} days\n", time / 10_000.0);
        }
    }
}
