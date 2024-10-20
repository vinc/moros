mod cmos;
mod rtc;
mod sleep;
mod timer;

pub use cmos::CMOS;
pub use rtc::RTC;
pub use sleep::{sleep, nanowait, halt};
pub use timer::{
    ticks, time_between_ticks, pit_frequency, set_pit_frequency_divider,
    last_rtc_update
};

use rtc::{Interrupt, Register, RTC_CENTURY};
use timer::{rdtsc, CLOCKS_PER_NANOSECOND};

pub fn init() {
    timer::init();
}
