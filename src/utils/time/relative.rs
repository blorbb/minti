use std::time::Duration;

use chrono::{prelude::*, Days};

pub fn get_next_occurrence(target_time: NaiveTime) -> DateTime<Local> {
    let current_time = Local::now().time();

    if current_time < target_time {
        // same day
        Local::now()
            .with_hour(target_time.hour())
            .unwrap()
            .with_minute(target_time.minute())
            .unwrap()
            .with_second(target_time.second())
            .unwrap()
    } else {
        // next day
        Local::now()
            .checked_add_days(Days::new(1))
            .unwrap()
            .with_hour(target_time.hour())
            .unwrap()
            .with_minute(target_time.minute())
            .unwrap()
            .with_second(target_time.second())
            .unwrap()
    }
}

pub fn duration_until_time(target_time: NaiveTime) -> Duration {
    let chrono_duration = get_next_occurrence(target_time) - Local::now();
    chrono_duration
        .to_std()
        .expect("Next occurrence is always after now")
}
