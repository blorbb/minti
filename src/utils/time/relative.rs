use std::ops::Add;

use time::{ext::NumericalDuration, Duration, OffsetDateTime, Time};

/// Calculates the next `OffsetDateTime` with the specified `target_time`
/// that is closest to the current datetime.
///
/// The `OffsetDateTime` will be on the same day if it is not `target_time` yet,
/// otherwise it will be on the next day.
///
/// If it is currently the exact `target_time`, the next day will be returned.
///
/// # Panics
/// Panics if the local time cannot be determined.
pub fn get_next_occurrence(target_time: Time) -> OffsetDateTime {
    let current_time = OffsetDateTime::now_local().unwrap().time();

    if current_time < target_time {
        // same day
        OffsetDateTime::now_local()
            .unwrap()
            .replace_time(target_time)
    } else {
        // next day
        OffsetDateTime::now_local()
            .unwrap()
            .add(1.days())
            .replace_time(target_time)
    }
}

/// Calculates the duration until the next `OffsetDateTime` with the specified
/// `target_time`.
///
/// The duration will always be less than or equal to 24 hours.
///
/// # Panics
/// Panics if the local time cannot be determined.
pub fn duration_until_time(target_time: Time) -> Duration {
    get_next_occurrence(target_time) - OffsetDateTime::now_local().unwrap()
}
