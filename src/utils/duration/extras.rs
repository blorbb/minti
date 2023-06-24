use super::units;
use std::time::Duration;

pub trait DurationUtils {
    fn from_mins(mins: u64) -> Duration;

    fn from_hours(hours: u64) -> Duration;

    fn from_days(days: u64) -> Duration;

    fn as_mins(&self) -> u64;

    fn as_hours(&self) -> u64;

    fn as_days(&self) -> u64;

    fn from_millis_f64(millis: f64) -> Duration;

    fn from_mins_f64(mins: f64) -> Duration;

    fn from_hours_f64(hours: f64) -> Duration;

    fn from_days_f64(days: f64) -> Duration;
}

impl DurationUtils for Duration {
    fn from_mins(mins: u64) -> Duration {
        Self::from_secs(mins * units::SECS_IN_MIN)
    }

    fn from_hours(hours: u64) -> Duration {
        Self::from_secs(hours * units::SECS_IN_HOUR)
    }

    fn from_days(days: u64) -> Duration {
        Self::from_secs(days * units::SECS_IN_DAY)
    }

    fn as_mins(&self) -> u64 {
        self.as_secs() / units::SECS_IN_MIN
    }

    fn as_hours(&self) -> u64 {
        self.as_secs() / units::SECS_IN_HOUR
    }

    fn as_days(&self) -> u64 {
        self.as_secs() / units::SECS_IN_DAY
    }

    fn from_millis_f64(millis: f64) -> Duration {
        Self::from_secs_f64(millis / units::MILLIS_IN_SEC as f64)
    }

    fn from_mins_f64(mins: f64) -> Duration {
        Self::from_secs_f64(mins * units::SECS_IN_MIN as f64)
    }

    fn from_hours_f64(hours: f64) -> Duration {
        Self::from_secs_f64(hours * units::SECS_IN_HOUR as f64)
    }

    fn from_days_f64(days: f64) -> Duration {
        Self::from_secs_f64(days * units::SECS_IN_DAY as f64)
    }
}
