pub mod errors;
mod tokens;
mod unparsed_tokens;

use std::time::Duration;

use self::errors::ParseError;

pub fn parse_input(input: &str) -> Result<Duration, ParseError> {
    let tokens = unparsed_tokens::build_str_tokens(input)?;
    let tokens = tokens::parse_str_tokens(tokens)?;
    if tokens.is_empty() {
        return Err(ParseError::Empty);
    };
    let format = tokens::get_tokens_format(&tokens);
    tokens::parse_tokens(&tokens, &format)
}

#[cfg(test)]
mod tests {
    use crate::utils::duration::extras::DurationUtils;
    use std::time::Duration;

    use super::*;

    #[test]
    fn plain_int_as_mins() {
        assert_eq!(parse_input("23"), Ok(Duration::from_mins(23)));
        assert_eq!(parse_input("938"), Ok(Duration::from_mins(938)));
        assert_eq!(parse_input("0"), Ok(Duration::from_mins(0)));
    }

    mod units {
        use super::*;

        #[test]
        fn single_units() {
            assert_eq!(parse_input("3h"), Ok(Duration::from_hours(3)));
            assert_eq!(parse_input("10 h"), Ok(Duration::from_hours(10)));
            assert_eq!(parse_input("1.61 h"), Ok(Duration::from_hours_f64(1.61)));
            assert_eq!(parse_input("2 hours"), Ok(Duration::from_hours_f64(2.0)));

            assert_eq!(parse_input("3m"), Ok(Duration::from_mins(3)));
            assert_eq!(parse_input("49ms"), Ok(Duration::from_millis(49)));
        }

        #[test]
        fn multiple_units() {
            assert_eq!(
                parse_input("3h21m"),
                Ok(Duration::from_hours(3) + Duration::from_mins(21))
            );

            assert_eq!(
                parse_input("8d 23h 12m 5s 91ms"),
                Ok(Duration::from_days(8)
                    + Duration::from_hours(23)
                    + Duration::from_mins(12)
                    + Duration::from_secs(5)
                    + Duration::from_millis(91))
            )
        }

        #[test]
        fn trailing_number() {
            assert_eq!(
                parse_input("3h4"),
                Ok(Duration::from_hours(3) + Duration::from_mins(4))
            );

            assert_eq!(
                parse_input("3d 23h 12.3m 2"),
                Ok(Duration::from_days(3)
                    + Duration::from_hours(23)
                    + Duration::from_mins_f64(12.3)
                    + Duration::from_secs(2))
            )
        }
    }

    mod time {
        use chrono::NaiveTime;

        use crate::utils::time::relative::duration_until_time;

        use super::*;

        #[test]
        fn specific_12h_time() {
            assert_eq!(
                parse_input("3pm").unwrap().as_secs(),
                duration_until_time(NaiveTime::from_hms_opt(3 + 12, 0, 0).unwrap()).as_secs()
            );

            assert_eq!(
                parse_input("3:12pm").unwrap().as_secs(),
                duration_until_time(NaiveTime::from_hms_opt(3 + 12, 12, 0).unwrap()).as_secs()
            );

            assert_eq!(
                parse_input("5:12:30 am").unwrap().as_secs(),
                duration_until_time(NaiveTime::from_hms_opt(5, 12, 30).unwrap()).as_secs()
            );
        }
    }

    mod errors {
        use super::*;

        fn all_errors_with(error: ParseError, values: &[&str]) {
            for value in values {
                assert_eq!(
                    parse_input(value),
                    Err(error.clone()),
                    "{value} should have been an Empty Err"
                )
            }
        }

        fn all_errors(values: &[&str]) {
            for value in values {
                assert!(
                    parse_input(value).is_err(),
                    "{value} should have been an Err."
                )
            }
        }

        #[test]
        fn no_numbers() {
            all_errors_with(ParseError::Empty, &["", "h"]);
        }

        #[test]
        fn some_error() {
            all_errors(&["3.24x", "abc", "3:5:6:2:1"])
        }
    }
}
