use std::time::Duration;

use chrono::NaiveTime;

use crate::utils::{
    duration::extras::DurationUtils,
    time::{
        meridiem::{self, Meridiem},
        relative,
    },
};

use super::{
    errors::ParseError,
    structs::{Token, TokensFormat},
};

pub(super) fn parse_tokens(tokens: &Vec<Token>) -> Result<Duration, ParseError> {
    let format = get_tokens_format(tokens);

    match format {
        TokensFormat::SingleNumber => parse_single_number_tokens(tokens),
        TokensFormat::Time => parse_time_tokens(tokens),
        TokensFormat::Units => parse_unit_tokens(tokens),
    }
}

/// Tries to find the input format of the given list of tokens.
fn get_tokens_format(tokens: &Vec<Token>) -> TokensFormat {
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

/// Tries to parse a token list as a single number.
fn parse_single_number_tokens(tokens: &Vec<Token>) -> Result<Duration, ParseError> {
    let Token::Number(n) = tokens[0] else {
        return Err(ParseError::Empty);
    };
    Ok(Duration::from_mins_f64(n))
}

/// Tries to parse a token list as a specific time,
/// in 12h or 24h format.
fn parse_time_tokens(tokens: &Vec<Token>) -> Result<Duration, ParseError> {
    let mut meridiem: Option<Meridiem> = None;
    let mut time_sections = [0, 0, 0];
    // 0 = hour, 1 = min, 2 = sec
    let mut current_unit = 0;

    for token in tokens {
        // There should be a maximum of one am/pm
        // This would not run if it was set on the last iteration
        if meridiem.is_some() {
            return Err(ParseError::Unknown);
        };

        match token {
            Token::Separator => {
                current_unit += 1;
                // check needs to be here to avoid an index error
                if current_unit > 2 {
                    return Err(ParseError::TooManySeparators);
                }
            }
            // only allow times with integers
            Token::Number(n) if n.fract() == 0.0 => time_sections[current_unit] = *n as u32,
            Token::Number(n) => return Err(ParseError::InvalidNumber(n.to_string())),
            Token::Meridiem(m) => meridiem = Some(*m),
            _ => return Err(ParseError::ClashingFormats),
        }
    }

    let end_time = match meridiem {
        Some(meri) => {
            meridiem::new_12h_time(time_sections[0], time_sections[1], time_sections[2], meri)
        }
        None => NaiveTime::from_hms_opt(time_sections[0], time_sections[1], time_sections[2]),
    }
    .ok_or(ParseError::Unknown)?;

    Ok(relative::duration_until_time(end_time))
}

/// Tries to parse a token list as a duration with units.
fn parse_unit_tokens(tokens: &Vec<Token>) -> Result<Duration, ParseError> {
    let mut total_duration = Duration::ZERO;
    let mut current_number = 0.0;

    for token in tokens {
        match token {
            Token::Number(n) => current_number = *n,
            Token::Unit(u) => total_duration += u.to_duration(current_number),
            _ => return Err(ParseError::ClashingFormats),
        }
    }

    // the above only adds a number to the total when a unit is encountered
    // add the trailing number if it exists, as there is no unit after
    if let Token::Number(n) = tokens.last().unwrap() {
        let Token::Unit(unit) = tokens[tokens.len() - 2] else {
            return Err(ParseError::SmallerThanMilli(*n));
        };

        total_duration += unit
            .smaller_unit()
            .ok_or(ParseError::SmallerThanMilli(*n))?
            .to_duration(*n);
    };

    Ok(total_duration)
}
