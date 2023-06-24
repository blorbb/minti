use std::str::FromStr;

use chrono::NaiveTime;

use crate::utils::parse::errors::ParseError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Meridiem {
    Ante,
    Post,
}

impl Meridiem {
    pub const AM_TOKENS: [&str; 2] = ["am", "a.m."];
    pub const PM_TOKENS: [&str; 2] = ["pm", "p.m."];
}

impl FromStr for Meridiem {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            s if Self::AM_TOKENS.contains(&s) => Self::Ante,
            s if Self::PM_TOKENS.contains(&s) => Self::Post,
            s => Err(Self::Err::InvalidUnit(s.to_string()))?,
        })
    }
}

/// Makes a new `NaiveTime` from 12 hour notation.
/// - 12am becomes 00:00:00.
/// - 12pm becomes 12:00:00.
/// - All other times are as expected.
///
/// Follows the same restrictions as `NaiveTime::from_hms_opt`,
/// but the `hour` also cannot be greater than 12.
///
/// Returns `None` on invalid hour, minute and/or second.
pub const fn new_12h_time(hour: u32, min: u32, sec: u32, meridiem: Meridiem) -> Option<NaiveTime> {
    let hour_12 = match meridiem {
        _ if hour > 12 => return None,
        // 12am becomes 00:00
        Meridiem::Ante if hour == 12 => 0,
        Meridiem::Ante => hour,
        // 12pm stays as 12:00
        Meridiem::Post if hour == 12 => 12,
        Meridiem::Post => hour + 12,
    };
    NaiveTime::from_hms_opt(hour_12, min, sec)
}
