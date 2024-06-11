use std::str::FromStr;

use time::Time;

use crate::interpreter;

/// An enum representing either AM or PM.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Meridiem {
    Ante,
    Post,
}

impl Meridiem {
    pub const AM_TOKENS: [&'static str; 2] = ["am", "a.m."];
    pub const PM_TOKENS: [&'static str; 2] = ["pm", "p.m."];
}

impl FromStr for Meridiem {
    type Err = interpreter::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            s if Self::AM_TOKENS.contains(&s) => Self::Ante,
            s if Self::PM_TOKENS.contains(&s) => Self::Post,
            s => Err(Self::Err::InvalidUnit(s.to_string()))?,
        })
    }
}

/// Makes a new `Time` from 12 hour notation.
/// - 12am becomes 00:00:00.
/// - 12pm becomes 12:00:00.
/// - 0 hours is the same as 12.
/// - All other times are as expected.
///
/// Follows the same restrictions as `Time::from_hms`,
/// but the `hour` also cannot be greater than 12.
///
/// Returns `None` on invalid hour, minute and/or second.
///
/// # Examples
/// ```rust
/// use minti_ui::time::meridiem::{
///     new_12h_time,
///     Meridiem::{Ante, Post},
/// };
/// use time::Time;
///
/// assert_eq!(new_12h_time(5, 30, 0, Ante), Time::from_hms(5, 30, 0).ok());
/// assert_eq!(new_12h_time(5, 30, 0, Post), Time::from_hms(5+12, 30, 0).ok());
/// assert_eq!(new_12h_time(12, 10, 0, Ante), Time::from_hms(0, 10, 0).ok());
/// assert_eq!(new_12h_time(12, 0, 0, Post), Time::from_hms(12, 0, 0).ok());
/// assert!(new_12h_time(13, 10, 0, Ante).is_none())
/// ```
pub fn new_12h_time(hour: u8, min: u8, sec: u8, meridiem: Meridiem) -> Option<Time> {
    let hour_12 = match meridiem {
        _ if hour > 12 => return None,
        // 12am becomes 00:00
        Meridiem::Ante if hour == 12 => 0,
        Meridiem::Ante => hour,
        // 12pm stays as 12:00
        Meridiem::Post if hour == 12 => 12,
        Meridiem::Post => hour + 12,
    };
    Time::from_hms(hour_12, min, sec).ok()
}
