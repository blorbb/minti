use std::str::FromStr;

use std::time::Duration;

use crate::utils::parse::errors::ParseError;

use super::extras::DurationUtils;

pub const MILLIS_IN_SEC: u64 = 1000;
pub const SECS_IN_MIN: u64 = 60;
pub const MINS_IN_HOUR: u64 = 60;
pub const HOURS_IN_DAY: u64 = 24;
pub const SECS_IN_HOUR: u64 = SECS_IN_MIN * MINS_IN_HOUR;
pub const SECS_IN_DAY: u64 = SECS_IN_HOUR * HOURS_IN_DAY;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimeUnit {
    Milli,
    Sec,
    Min,
    Hour,
    Day,
}

impl TimeUnit {
    pub const MILLI_TOKENS: [&str; 7] = [
        "ms",
        "milli",
        "millis",
        "millisec",
        "millisecs",
        "millisecond",
        "milliseconds",
    ];
    pub const SEC_TOKENS: [&str; 5] = ["s", "sec", "secs", "second", "seconds"];
    pub const MIN_TOKENS: [&str; 5] = ["m", "min", "mins", "minute", "minutes"];
    pub const HOUR_TOKENS: [&str; 5] = ["h", "hr", "hrs", "hour", "hours"];
    pub const DAY_TOKENS: [&str; 3] = ["d", "day", "days"];

    pub fn numeric_value(&self) -> u8 {
        match self {
            TimeUnit::Milli => 0,
            TimeUnit::Sec => 1,
            TimeUnit::Min => 2,
            TimeUnit::Hour => 3,
            TimeUnit::Day => 4,
        }
    }

    pub fn number_to_variant(num: u8) -> Option<Self> {
        if num == 0 {
            Some(Self::Milli)
        } else if num == 1 {
            Some(Self::Sec)
        } else if num == 2 {
            Some(Self::Min)
        } else if num == 3 {
            Some(Self::Hour)
        } else if num == 4 {
            Some(Self::Day)
        } else {
            None
        }
    }

    pub fn larger_unit(&self) -> Option<Self> {
        Self::number_to_variant(self.numeric_value().saturating_add(1))
    }

    pub fn smaller_unit(&self) -> Option<Self> {
        Self::number_to_variant(self.numeric_value().wrapping_sub(1))
    }

    pub fn to_duration(&self, value: f64) -> Duration {
        match self {
            TimeUnit::Milli => Duration::from_millis_f64(value),
            TimeUnit::Sec => Duration::from_secs_f64(value),
            TimeUnit::Min => Duration::from_mins_f64(value),
            TimeUnit::Hour => Duration::from_hours_f64(value),
            TimeUnit::Day => Duration::from_days_f64(value),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            s if Self::MILLI_TOKENS.contains(&s) => Self::Milli,
            s if Self::SEC_TOKENS.contains(&s) => Self::Sec,
            s if Self::MIN_TOKENS.contains(&s) => Self::Min,
            s if Self::HOUR_TOKENS.contains(&s) => Self::Hour,
            s if Self::DAY_TOKENS.contains(&s) => Self::Day,
            s => Err(Self::Err::InvalidUnit(s.to_string()))?,
        })
    }
}
