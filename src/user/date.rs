use crate::{print, kernel, user};
use crate::kernel::cmos::CMOS;

pub fn print_time_in_seconds(time: f64) {
    if time < 1.0e3 {
        print!("{:.3} seconds\n", time);
    } else if time < 1.0e6 {
        print!("{:.3} kiloseconds\n", time / 1.0e3);
    } else if time < 1.0e9 {
        print!("{:.3} megaseconds\n", time / 1.0e6);
    } else {
        print!("{:.3} gigaseconds\n", time / 1.0e9);
    }
}

pub fn print_time_in_days(time: f64) {
    if time < 0.01 {
        print!("{:.2} dimidays\n", time * 10_000.0);
    } else if time < 1.0 {
        print!("{:.2} centidays\n", time * 100.0);
    } else {
        print!("{:.2} days\n", time);
    }
}

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 2 && args[1] == "--raw" {
        print!("{:.6}\n", kernel::clock::clock_realtime());
    } else if args.len() == 2 && args[1] == "--metric" {
        print_time_in_seconds(kernel::clock::clock_realtime());
    } else if args.len() == 2 && args[1] == "--iso-8601" {
        let rtc = CMOS::new().rtc();
        print!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}\n",
            rtc.year, rtc.month, rtc.day,
            rtc.hour, rtc.minute, rtc.second
        );
    } else {
        let rtc = CMOS::new().rtc();
        let seconds_since_midnight = rtc.hour as f64 * 3600.0
                                   + rtc.minute as f64 * 60.0
                                   + rtc.second as f64;
        print_time_in_days(seconds_since_midnight / 86400.0);
    }
    user::shell::ExitCode::CommandSuccessful
}
