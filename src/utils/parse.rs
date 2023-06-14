use std::{mem, time::Duration};

use super::duration::{extras::DurationUtils, units::TimeUnit};

pub fn parse_input(input: &str) -> Result<Duration, ParseError> {
    let tokens = build_str_tokens(input)?;
    let tokens = parse_str_tokens(tokens)?;
    validate_tokens(&tokens)?;
    Ok(parse_tokens(&tokens, get_tokens_format(&tokens)))
}

fn build_str_tokens(input: &str) -> Result<Vec<(StrType, String)>, ParseError> {
    let input = input.to_lowercase().replace(' ', "");

    let mut is_first_char = true;
    let mut token_list: Vec<(StrType, String)> = vec![];
    let mut previous_token_type = StrType::Separator; // will be overwritten
    let mut current_token = "".to_string();

    for ch in input.chars() {
        let char_token_type = StrType::try_from(ch)?;

        // push previous token and start a new one
        if char_token_type != previous_token_type
            || is_first_char
            // always make it a new token if its a separator
            // so `1::2` is the same as `1:0:2`.
            || char_token_type == StrType::Separator
        {
            let complete_token = mem::take(&mut current_token);
            token_list.push((previous_token_type, complete_token));

            previous_token_type = char_token_type;
            is_first_char = false;
        };

        // add character to the current token
        current_token.push(ch);
    }

    let complete_token = mem::take(&mut current_token);
    token_list.push((previous_token_type, complete_token)); // include the last token

    // first element is an empty string
    token_list.remove(0);

    Ok(token_list)
}

fn parse_str_tokens(tokens: Vec<(StrType, String)>) -> Result<Vec<Token>, ParseError> {
    tokens
        .into_iter()
        .map(|(t, string)| -> Result<_, _> {
            Ok(match t {
                StrType::Number => Token::Number(
                    string
                        .parse()
                        .map_err(|_| ParseError::InvalidNumber(string))?,
                ),
                StrType::Unit => Token::Unit(string.parse::<TimeUnit>()?),
                StrType::Separator => Token::Separator,
            })
        })
        .collect::<Result<_, _>>()
}

fn validate_tokens(tokens: &Vec<Token>) -> Result<(), ParseError> {
    // tokens either has separators or units, not both
    if tokens.contains(&Token::Separator) && tokens.iter().any(|v| matches!(v, Token::Unit(_))) {
        Err(ParseError::ClashingFormats)

    // if the tokens end in a <unit><number>, unit must not be `ms`.
    // the units of <number> are assumed to be 1 unit lower than the preceding <unit>
    } else if tokens.len() > 1
        && let Some(Token::Number(n)) = tokens.last()
        && tokens.get(tokens.len() - 2) == Some(&Token::Unit(TimeUnit::Milli))
    {
        Err(ParseError::SmallerThanMilli(*n))

    // check that there are up to 3 separators
    }  else if tokens.iter().filter(|v| v == &&Token::Separator).count() > 3 {
        Err(ParseError::TooManySeparators)

    // check that there is a number
    } else if !tokens.iter().any(|t| matches!(t, Token::Number(_))) {
        Err(ParseError::Empty)

    // check for NaNs
    } else if tokens.iter().any(|t| matches!(t, Token::Number(n) if n.is_infinite() || n.is_nan())) {
        Err(ParseError::NaN)
    } else {
        Ok(())
    }
}

/// Tokens are assumed to be previously validated.
fn get_tokens_format(tokens: &Vec<Token>) -> TokensFormat {
    if tokens.len() == 1 {
        TokensFormat::SingleNumber
    } else if tokens.contains(&Token::Separator) {
        TokensFormat::Separators
    } else {
        TokensFormat::Units
    }
}

/// tokens are assumed to be previously validated.
fn parse_tokens(tokens: &Vec<Token>, method: TokensFormat) -> Duration {
    match method {
        TokensFormat::SingleNumber => {
            let Token::Number(n) = tokens[0] else {
                panic!("Tokens were not validated properly, {tokens:?} was parsed as a single-number variant")
            };
            Duration::from_mins_f64(n)
        }
        TokensFormat::Separators => {
            let mut current_unit = TimeUnit::Sec;
            let mut total_duration = Duration::ZERO;
            for token in tokens.iter().rev() {
                if token == &Token::Separator {
                    current_unit = current_unit
                        .larger_unit()
                        .expect("Units should not go out of bounds");
                } else if let Token::Number(n) = token {
                    total_duration += current_unit.to_duration(*n)
                } else {
                    panic!("Tokens were not validated properly, {tokens:?} has a non-numeric or separator token")
                }
            }
            total_duration
        }
        TokensFormat::Units => {
            let mut total_duration = Duration::ZERO;
            let mut current_number = 0.0;

            for token in tokens {
                if let Token::Number(n) = token {
                    current_number = *n;
                } else if let Token::Unit(unit) = token {
                    total_duration += unit.to_duration(current_number)
                } else {
                    panic!("Tokens were not validated properly, {tokens:?} has a separator token")
                };
            }

            // add the trailing number if it exists
            if let Token::Number(n) = tokens.last().unwrap() {
                let Token::Unit(unit) = tokens[tokens.len() - 2] else {
                    panic!("Tokens {tokens:?} do not end with <unit><num>")
                };
                total_duration += unit.smaller_unit().unwrap().to_duration(*n);
            };

            total_duration
        }
    }
}

enum TokensFormat {
    SingleNumber,
    Separators,
    Units,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    NaN,
    InvalidCharacter(char),
    InvalidNumber(String),
    InvalidUnit(String),
    SmallerThanMilli(f64),
    ClashingFormats,
    TooManySeparators,
    Empty,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum StrType {
    Number,
    Unit,
    Separator,
}

impl TryFrom<char> for StrType {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value.is_ascii_alphabetic() {
            Ok(StrType::Unit)
        } else if value.is_ascii_digit() || value == '.' {
            Ok(StrType::Number)
        } else if value == ':' {
            Ok(StrType::Separator)
        } else {
            Err(ParseError::InvalidCharacter(value))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Token {
    Number(f64),
    Unit(TimeUnit),
    Separator,
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

    mod separators {
        use super::*;

        #[test]
        fn one_separator_as_mm_ss() {
            assert_eq!(
                parse_input("12:30"),
                Ok(Duration::from_mins(12) + Duration::from_secs(30))
            );
        }

        #[test]
        fn two_separators_as_hh_mm_ss() {
            assert_eq!(
                parse_input("1:2:3"),
                Ok(Duration::from_hours(1) + Duration::from_mins(2) + Duration::from_secs(3))
            );
        }

        #[test]
        fn three_separators_as_dd_hh_mm_ss() {
            assert_eq!(
                parse_input("1:23:59:0"),
                Ok(Duration::from_days(1)
                    + Duration::from_hours(23)
                    + Duration::from_mins(59)
                    + Duration::from_secs(0))
            )
        }
    }

    mod errors {
        use super::*;

        fn all_errors_with(error: ParseError, values: &[&str]) {
            for value in values {
                assert_eq!(parse_input(value), Err(error.clone()))
            }
        }

        fn all_errors(values: &[&str]) {
            for value in values {
                assert!(parse_input(value).is_err(), "{value} failed the test.")
            }
        }

        #[test]
        fn no_numbers() {
            all_errors_with(ParseError::Empty, &["", ":", "::", "h"]);
        }

        #[test]
        fn some_error() {
            all_errors(&["3.24x", "abc", "3:5:6:2:1"])
        }
    }

    mod private {
        use super::*;

        mod build_tokens {
            use super::*;

            #[test]
            fn separate_time_unit() {
                assert_eq!(
                    build_str_tokens("1d"),
                    Ok(vec![
                        (StrType::Number, "1".to_string()),
                        (StrType::Unit, "d".to_string())
                    ])
                );
                assert_eq!(
                    build_str_tokens("1.3 h"),
                    Ok(vec![
                        (StrType::Number, "1.3".to_string()),
                        (StrType::Unit, "h".to_string())
                    ])
                );
                assert_eq!(
                    build_str_tokens("3M "),
                    Ok(vec![
                        (StrType::Number, "3".to_string()),
                        (StrType::Unit, "m".to_string())
                    ])
                );
                assert_eq!(
                    build_str_tokens("94 ms"),
                    Ok(vec![
                        (StrType::Number, "94".to_string()),
                        (StrType::Unit, "ms".to_string())
                    ])
                );
            }

            #[test]
            fn separate_multiple_time_unit() {
                assert_eq!(
                    build_str_tokens("1d3h"),
                    Ok(vec![
                        (StrType::Number, "1".to_string()),
                        (StrType::Unit, "d".to_string()),
                        (StrType::Number, "3".to_string()),
                        (StrType::Unit, "h".to_string())
                    ])
                );
                assert_eq!(
                    build_str_tokens("5h 92m 1ms"),
                    Ok(vec![
                        (StrType::Number, "5".to_string()),
                        (StrType::Unit, "h".to_string()),
                        (StrType::Number, "92".to_string()),
                        (StrType::Unit, "m".to_string()),
                        (StrType::Number, "1".to_string()),
                        (StrType::Unit, "ms".to_string())
                    ])
                );
            }

            #[test]
            fn separate_separators() {
                assert_eq!(
                    build_str_tokens("3:4:7"),
                    Ok(vec![
                        (StrType::Number, "3".to_string()),
                        (StrType::Separator, ":".to_string()),
                        (StrType::Number, "4".to_string()),
                        (StrType::Separator, ":".to_string()),
                        (StrType::Number, "7".to_string()),
                    ])
                );
                assert_eq!(
                    build_str_tokens("1::2"),
                    Ok(vec![
                        (StrType::Number, "1".to_string()),
                        (StrType::Separator, ":".to_string()),
                        (StrType::Separator, ":".to_string()),
                        (StrType::Number, "2".to_string()),
                    ])
                );
            }
        }
    }
}
