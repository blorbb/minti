use std::time::Duration;

use chrono::NaiveTime;

use crate::utils::{
    duration::{extras::DurationUtils, units::TimeUnit},
    time::{meridiem::Meridiem, relative},
};

use super::{
    errors::ParseError,
    unparsed_tokens::{UnparsedToken, UnparsedTokenType},
};

pub(super) fn parse_str_tokens(tokens: Vec<UnparsedToken>) -> Result<Vec<Token>, ParseError> {
    tokens
        .into_iter()
        .map(Token::try_from)
        .collect::<Result<_, _>>()
}

/// Tries to find the input format of the given list of tokens.
pub(super) fn get_tokens_format(tokens: &Vec<Token>) -> TokensFormat {
    if tokens.len() == 1 {
        TokensFormat::SingleNumber
    } else if tokens.contains(&Token::Separator)
        || tokens.iter().any(|t| matches!(t, Token::Meridiem(_)))
    {
        TokensFormat::Time
    } else {
        TokensFormat::Units
    }
}

pub(super) fn parse_tokens(
    tokens: &Vec<Token>,
    format: &TokensFormat,
) -> Result<Duration, ParseError> {
    match format {
        TokensFormat::SingleNumber => {
            let Token::Number(n) = tokens[0] else {
                return Err(ParseError::Empty);
            };
            Ok(Duration::from_mins_f64(n))
        }
        TokensFormat::Time => {
            let mut meridiem: Option<Meridiem> = None;
            let mut time_sections = [0, 0, 0];
            // 0 = hour
            // 1 = min
            // 2 = sec
            let mut current_unit = 0;
            for token in tokens {
                // There should be a maximum of one am/pm
                // This would not run if it was set on the last iteration
                if meridiem.is_some() {
                    return Err(ParseError::Unknown);
                };

                if token == &Token::Separator {
                    // move to the next unit
                    current_unit += 1;
                    if current_unit > 2 {
                        return Err(ParseError::TooManySeparators);
                    }
                } else if let Token::Number(n) = token {
                    // add the number to the corresponding unit

                    if n.fract() != 0.0 {
                        return Err(ParseError::InvalidNumber(n.to_string()));
                    };

                    time_sections[current_unit] = *n as u32;
                } else if let Token::Meridiem(m) = token {
                    meridiem = Some(*m);
                } else {
                    return Err(ParseError::ClashingFormats);
                };
            }

            // set 12am to 0
            if meridiem == Some(Meridiem::Ante) && time_sections[0] == 12 {
                time_sections[0] = 0;
            };
            // 12pm stays as 12, everything else adds 12h
            if meridiem == Some(Meridiem::Post) && time_sections[0] != 12 {
                time_sections[0] += 12;
            };

            let end_time =
                NaiveTime::from_hms_opt(time_sections[0], time_sections[1], time_sections[2])
                    .ok_or(ParseError::Unknown)?;

            Ok(relative::duration_until_time(end_time))
        }
        TokensFormat::Units => {
            let mut total_duration = Duration::ZERO;
            let mut current_number = 0.0;

            for token in tokens {
                if let Token::Number(n) = token {
                    current_number = *n;
                } else if let Token::Unit(unit) = token {
                    total_duration += unit.to_duration(current_number);
                } else {
                    return Err(ParseError::ClashingFormats);
                };
            }

            // add the trailing number if it exists
            if let Token::Number(n) = tokens.last().unwrap() {
                let Token::Unit(unit) = tokens[tokens.len() - 2] else {
                    return Err(ParseError::SmallerThanMilli(*n));
                };
                total_duration += unit.smaller_unit().unwrap().to_duration(*n);
            };

            Ok(total_duration)
        }
    }
}

pub(super) enum TokensFormat {
    /// Checks that the length of a Vec<Token> is 1.
    /// Does not check that it is a number.
    SingleNumber,
    /// Checks that the Vec<Token> has a separator ":",
    /// or has am/pm.
    Time,
    /// If none of the other formats have been matched.
    Units,
}

/// Guarantees:
/// - Number is a valid float, not NaN or infinity.
/// - Text is valid, either a time unit or meridiem.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Token {
    Number(f64),
    Unit(TimeUnit),
    Meridiem(Meridiem),
    Separator,
}

impl TryFrom<UnparsedToken> for Token {
    type Error = ParseError;

    fn try_from(value: UnparsedToken) -> Result<Self, Self::Error> {
        let token = value.variant;
        let string = value.string;
        Ok(match token {
            UnparsedTokenType::Number => {
                // TODO figure out how to remove the clone
                let num = string
                    .parse::<f64>()
                    .map_err(|_| ParseError::InvalidNumber(string.clone()))?;

                if num.is_nan() || num.is_infinite() {
                    return Err(ParseError::InvalidNumber(string));
                }
                Token::Number(num)
            }
            UnparsedTokenType::Text => {
                if let Ok(n) = string.parse::<TimeUnit>() {
                    Token::Unit(n)
                } else {
                    Token::Meridiem(string.parse::<Meridiem>()?)
                }
            }
            UnparsedTokenType::Separator => Token::Separator,
        })
    }
}
