mod cmos;
mod boot;
mod epoch;
mod rtc;
mod sync;
mod timer;

pub use boot::{boot_time, BootTime};
pub use epoch::{epoch_time, EpochTime};
pub use rtc::RTC;
pub use sync::{halt, sleep, wait};
pub use timer::{ticks, pit_frequency, set_pit_frequency};

use crate::api;

use alloc::string::String;
use time::{Duration, OffsetDateTime};

pub fn init() {
    timer::init();
}

/// Returns the current date and time.
pub fn date() -> String {
    let s = epoch::epoch_time();
    let ns = Duration::nanoseconds(
        libm::floor(1e9 * (s - libm::floor(s))) as i64
    );
    let dt = OffsetDateTime::from_unix_timestamp(s as i64) + ns;
    dt.format(api::clock::DATE_TIME_ZONE)
}
