pub mod errors;
mod parse_tokens;
mod structs;
mod unparsed_tokens;

use time::Duration;
use self::{errors::ParseError, structs::Token};

/// Tries to parse a user inputted string as a duration.
///
/// There are 2 main formats:
/// - A duration, specified with units like "1h 30m".
///     - The units accepted are days, hours, minutes, seconds and
///       milliseconds. Several different ways of writing each are accepted
///       (e.g. "h", "hrs", "hours").
///     - If no units are given, minutes is assumed.
///     - If the string ends in a number with no unit, it is assumed to be one
///       unit smaller than the previous (e.g. "2m 30" is the same as "2m 30s").
///     - Decimals are accepted, like "3.5h".
/// - A specific time, like "5:30pm". Finds the duration until the next
///   occurrence of the specified time.
///     - If "am" or "pm" is added, the duration until the next occurrence of
///       that time is returned.
///     - If no "am" or "pm" is added, it will be interpreted as the closest one
///       (e.g. at 2pm, "3:30" is the same as "3:30pm" and "1:30" is the same
///       as "1:30am").
///     - A no-meridiem time with only the hour time can be inputted by adding
///       a ":" (e.g. "3" is interpreted as 3 minutes while "3:" is interpreted
///       as 3 am/pm, whichever is closest).
///
/// # Errors
/// Errors if the input does not match any of the above formats.
///
/// The error reason will try to be given, however it may be inconsistent
/// and change if the implementation is modified.
///
/// # Examples
/// ```rust
/// use time::{Duration, ext::NumericalDuration};
/// # use minti_ui::utils::parse::parse_input;
///
/// assert_eq!(parse_input("3").unwrap(), 3.minutes());
/// assert_eq!(
///     parse_input("3h 20m 10").unwrap(),
///     3.hours() + 20.minutes() + 10.seconds()
/// );
/// ```
pub fn parse_input(input: &str) -> Result<Duration, ParseError> {
    let tokens = unparsed_tokens::build_unparsed_tokens(input)?;
    let tokens: Vec<Token> = tokens
        .into_iter()
        .map(Token::try_from)
        .collect::<Result<_, _>>()?;

    if tokens.is_empty() {
        return Err(ParseError::Empty);
    };

    parse_tokens::parse_tokens(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::ext::NumericalDuration;

    #[test]
    fn plain_int_as_mins() {
        assert_eq!(parse_input("23").unwrap(), 23.minutes());
        assert_eq!(parse_input("938").unwrap(), 938.minutes());
        assert_eq!(parse_input("0").unwrap(), 0.minutes());
    }

    mod units {
        use super::*;

        #[test]
        fn single_units() {
            assert_eq!(parse_input("3h").unwrap(), 3.hours());
            assert_eq!(parse_input("10 h").unwrap(), 10.hours());
            assert_eq!(parse_input("1.61 h").unwrap(), 1.61.hours());
            assert_eq!(parse_input("2 hours").unwrap(), 2.hours());

            assert_eq!(parse_input("3m").unwrap(), 3.minutes());
            assert_eq!(parse_input("49ms").unwrap(), 49.milliseconds());
        }

        #[test]
        fn multiple_units() {
            assert_eq!(parse_input("3h21m").unwrap(), 3.hours() + 21.minutes());

            assert_eq!(
                parse_input("8d 23h 12m 5s 91ms").unwrap(),
                8.days() + 23.hours() + 12.minutes() + 5.seconds() + 91.milliseconds()
            )
        }

        #[test]
        fn trailing_number() {
            assert_eq!(parse_input("3h4").unwrap(), 3.hours() + 4.minutes());

            assert_eq!(
                parse_input("3d 23h 12.3m 2").unwrap(),
                3.days() + 23.hours() + 12.3.minutes() + 2.seconds()
            )
        }
    }

    mod times {
        use crate::utils::time::relative::duration_until_time;
        use time::Time;

        use super::*;

        #[test]
        fn specific_12h_time() {
            assert_eq!(
                parse_input("3pm").unwrap().whole_seconds(),
                duration_until_time(Time::from_hms(3 + 12, 0, 0).unwrap()).whole_seconds()
            );

            assert_eq!(
                parse_input("3:12pm").unwrap().whole_seconds(),
                duration_until_time(Time::from_hms(3 + 12, 12, 0).unwrap()).whole_seconds()
            );

            assert_eq!(
                parse_input("5:12:30 am").unwrap().whole_seconds(),
                duration_until_time(Time::from_hms(5, 12, 30).unwrap()).whole_seconds()
            );
        }
    }

    mod errors {
        use super::*;
        fn all_errors(values: &[&str]) {
            for value in values {
                assert!(
                    parse_input(value).is_err(),
                    "{value} should have been an Err."
                )
            }
        }

        #[test]
        fn raises_error() {
            all_errors(&[
                "3.24x",
                "abc",
                "3:5:6:2:1",
                "",
                "h",
                "10s 300ms 10",
                "13:0:0am",
                "3pm 10",
            ])
        }
    }
}
