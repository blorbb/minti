use time::OffsetDateTime;

const NANOS_IN_MILLI: i128 = 10i128.pow(6);

pub fn to_unix_millis(date: OffsetDateTime) -> i64 {
    (date.unix_timestamp_nanos() / NANOS_IN_MILLI) as i64
}

pub fn from_unix_millis(millis: i64) -> Result<OffsetDateTime, time::error::ComponentRange> {
    OffsetDateTime::from_unix_timestamp_nanos((millis as i128 * NANOS_IN_MILLI) as i128)
}
