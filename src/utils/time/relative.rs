use std::ops::Add;

use time::{ext::NumericalDuration, Duration, OffsetDateTime, Time};

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

pub fn duration_until_time(target_time: Time) -> Duration {
    get_next_occurrence(target_time) - OffsetDateTime::now_local().unwrap()
}
