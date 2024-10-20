mod cmos;
mod boot;
mod epoch;
mod rtc;
mod sleep;
mod timer;

pub use boot::{boot_time, BootTime}; // TODO: Rename to boot_time
pub use epoch::{epoch_time, EpochTime}; // TODO: Rename to epoch_time
pub use cmos::CMOS;
pub use rtc::RTC;
pub use sleep::{sleep, nanowait, halt};
pub use timer::{
    ticks, time_between_ticks, pit_frequency, set_pit_frequency_divider,
    last_rtc_update
};

use rtc::{Interrupt, Register, RTC_CENTURY};
use timer::{rdtsc, CLOCKS_PER_NANOSECOND};

use crate::api::clock::DATE_TIME_ZONE;

use time::{Duration, OffsetDateTime};

pub fn init() {
    timer::init();
}

pub fn log_rtc() {
    let s = epoch_time();
    let ns = Duration::nanoseconds(
        libm::floor(1e9 * (s - libm::floor(s))) as i64
    );
    let dt = OffsetDateTime::from_unix_timestamp(s as i64) + ns;
    let rtc = dt.format(DATE_TIME_ZONE);
    log!("RTC {}", rtc);
}
