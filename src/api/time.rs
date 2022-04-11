use crate::sys;
use crate::api::syscall;

use time::{OffsetDateTime, Duration, UtcOffset};

pub fn now() -> OffsetDateTime {
    let s = syscall::realtime(); // Since Unix Epoch
    let ns = Duration::nanoseconds(libm::floor(1e9 * (s - libm::floor(s))) as i64);
    OffsetDateTime::from_unix_timestamp(s as i64).to_offset(offset()) + ns
}

pub fn from_timestamp(ts: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(ts).to_offset(offset())
}

fn offset() -> UtcOffset {
    if let Some(tz) = sys::process::env("TZ") {
        if let Ok(offset) = tz.parse::<i32>() {
            return UtcOffset::seconds(offset);
        }
    }
    UtcOffset::seconds(0)
}
