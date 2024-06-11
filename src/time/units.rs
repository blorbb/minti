use std::str::FromStr;
use time::{ext::NumericalDuration, Duration};

use crate::interpreter;

pub const MILLIS_IN_SEC: u64 = 1000;
pub const SECS_IN_MIN: u64 = 60;
pub const MINS_IN_HOUR: u64 = 60;
pub const HOURS_IN_DAY: u64 = 24;
pub const SECS_IN_HOUR: u64 = SECS_IN_MIN * MINS_IN_HOUR;
pub const SECS_IN_DAY: u64 = SECS_IN_HOUR * HOURS_IN_DAY;

/// A representation of various units of duration.
///
/// Mainly used for converting string representations of units and getting
/// units relative to others.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TimeUnit {
    Milli,
    Sec,
    Min,
    Hour,
    Day,
}

impl TimeUnit {
    pub const MILLI_TOKENS: [&'static str; 7] = [
        "ms",
        "milli",
        "millis",
        "millisec",
        "millisecs",
        "millisecond",
        "milliseconds",
    ];
    pub const SEC_TOKENS: [&'static str; 5] = ["s", "sec", "secs", "second", "seconds"];
    pub const MIN_TOKENS: [&'static str; 5] = ["m", "min", "mins", "minute", "minutes"];
    pub const HOUR_TOKENS: [&'static str; 5] = ["h", "hr", "hrs", "hour", "hours"];
    pub const DAY_TOKENS: [&'static str; 3] = ["d", "day", "days"];

    /// Converts a unit to a number, which can be used to get other units
    /// relative to `self`.
    ///
    /// The number starts from milliseconds (0) to days (4).
    ///
    /// The number only makes sense when used in `number_to_variant`.
    #[must_use]
    const fn numeric_value(&self) -> u8 {
        match self {
            Self::Milli => 0,
            Self::Sec => 1,
            Self::Min => 2,
            Self::Hour => 3,
            Self::Day => 4,
        }
    }

    /// Converts a number to one of the unit variants.
    ///
    /// Only makes sense when used with the number returned in `numeric_value`.
    ///
    /// Returns `None` if `num` does not map to one of the variants (must be
    /// in range `0..=4`).
    #[must_use]
    const fn number_to_variant(num: u8) -> Option<Self> {
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

    /// Gets the unit that is one unit larger than `self`.
    ///
    /// Returns `None` if the unit is already the largest (Day).
    ///
    /// # Example
    /// ```rust
    /// # use minti_ui::utils::time::units::TimeUnit;
    ///
    /// assert_eq!(TimeUnit::Sec.larger_unit(), Some(TimeUnit::Min));
    /// assert_eq!(TimeUnit::Day.larger_unit(), None);
    /// ```
    pub const fn larger_unit(&self) -> Option<Self> {
        Self::number_to_variant(self.numeric_value().saturating_add(1))
    }

    /// Gets the unit that is one unit smaller than `self`.
    ///
    /// Returns `None` if the unit is already the smallest (Milli).
    ///
    /// # Example
    /// ```rust
    /// # use minti_ui::utils::time::units::TimeUnit;
    ///
    /// assert_eq!(TimeUnit::Hour.smaller_unit(), Some(TimeUnit::Min));
    /// assert_eq!(TimeUnit::Milli.smaller_unit(), None);
    /// ```
    pub const fn smaller_unit(&self) -> Option<Self> {
        Self::number_to_variant(self.numeric_value().wrapping_sub(1))
    }

    /// Converts a number to a duration using the unit of `self`.
    ///
    /// # Example
    /// ```rust
    /// # use minti_ui::utils::time::units::TimeUnit;
    /// use time::ext::NumericalDuration;
    ///
    /// assert_eq!(TimeUnit::Hour.to_duration(3.5), 3.5.hours());
    /// assert_eq!(TimeUnit::Sec.to_duration(20), 20.seconds());
    /// ```
    // TODO rename this
    pub fn to_duration(&self, value: f64) -> Duration {
        match self {
            Self::Milli => value.milliseconds(),
            Self::Sec => value.seconds(),
            Self::Min => value.minutes(),
            Self::Hour => value.hours(),
            Self::Day => value.days(),
        }
    }
}

impl FromStr for TimeUnit {
    type Err = interpreter::Error;

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
