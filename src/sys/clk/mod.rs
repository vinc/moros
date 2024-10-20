mod cmos;
mod rtc;

pub use cmos::CMOS;
pub use rtc::RTC;

use rtc::{Interrupt, Register, RTC_CENTURY};
