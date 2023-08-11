use az::SaturatingAs;
use time::OffsetDateTime;

const NANOS_IN_MILLI: i128 = 10i128.pow(6);

/// Gets the Unix timestamp in milliseconds.
pub fn to_unix_millis(date: OffsetDateTime) -> i64 {
    (date.unix_timestamp_nanos() / NANOS_IN_MILLI).saturating_as::<i64>()
}

// TODO figure out when this panics
/// Constructs an `OffsetDateTime` from the provided Unix timestamp in milliseconds.
pub fn from_unix_millis(millis: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(millis) * NANOS_IN_MILLI).unwrap()
}
