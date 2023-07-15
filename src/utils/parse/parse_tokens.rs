use az::SaturatingAs;
use time::{ext::NumericalDuration, Duration};

use crate::utils::time::{
    meridiem::{self, Meridiem},
    relative,
};

use super::{
    errors::ParseError,
    structs::{Token, TokensFormat},
};

/// Tries to parse a list of tokens to a duration.
///
/// # Errors
/// Errors if the list does not match any known format.
/// See `parse::parse_input` for more details on valid formats.
pub(super) fn parse_tokens(tokens: &[Token]) -> Result<Duration, ParseError> {
    log::trace!("parsing tokens");
    if tokens.is_empty() {
        log::trace!("no tokens found");
        return Err(ParseError::Empty);
    };

    let format = get_tokens_format(tokens);
    log::trace!("tokens are in {format:?} format");

    match format {
        TokensFormat::SingleNumber => parse_single_number_tokens(tokens),
        TokensFormat::Time => parse_time_tokens(tokens),
        TokensFormat::Units => parse_unit_tokens(tokens),
    }
}

/// Tries to find the input format of the given list of tokens.
fn get_tokens_format(tokens: &[Token]) -> TokensFormat {
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
fn parse_single_number_tokens(tokens: &[Token]) -> Result<Duration, ParseError> {
    let Token::Number(n) = tokens[0] else {
        log::trace!("single token is not a number");
        return Err(ParseError::Empty);
    };
    log::trace!("successfully parsed as {n} minutes");
    Ok(n.minutes())
}

/// Tries to parse a token list as a specific time,
/// in 12h or 24h format.
fn parse_time_tokens(tokens: &[Token]) -> Result<Duration, ParseError> {
    let mut meridiem: Option<Meridiem> = None;
    let mut time_sections: [u8; 3] = [0, 0, 0];
    // 0 = hour, 1 = min, 2 = sec
    let mut current_unit = 0;

    for token in tokens {
        log::trace!("parsing token {token:?}");

        // There should be a maximum of one am/pm
        // This would not run if it was set on the last iteration
        if meridiem.is_some() {
            log::trace!("found token after a meridiem");
            return Err(ParseError::Unknown);
        };

        #[expect(clippy::match_wildcard_for_single_variants)]
        match token {
            Token::Separator => {
                current_unit += 1;
                log::trace!("set time units to section {current_unit}");
                // check needs to be here to avoid an index error
                if current_unit > 2 {
                    log::trace!("found more than 2 separators");
                    return Err(ParseError::TooManySeparators);
                }
            }
            // only allow times with integers
            Token::Number(n) if n.fract() == 0.0 => {
                log::trace!("adding time {n} to section {current_unit}");
                time_sections[current_unit] = (*n).saturating_as::<u8>();
            }
            Token::Number(n) => {
                log::trace!("time {n} is not an integer");
                return Err(ParseError::InvalidNumber(n.to_string()));
            }
            Token::Meridiem(m) => {
                log::trace!("setting meridiem to {m:?}");
                meridiem = Some(*m);
            }
            _ => {
                log::trace!("token is not any accepted token in the time format");
                return Err(ParseError::ClashingFormats);
            }
        }
    }

    let [h, m, s] = time_sections;

    let duration = if let Some(meri) = meridiem {
        log::trace!("setting to closest {h}:{m}:{s} {meri:?}");
        relative::duration_until_time(
            meridiem::new_12h_time(h, m, s, meri).ok_or(ParseError::Unknown)?,
        )
    } else {
        log::trace!("setting to closest {h}:{m}:{s}");

        // find the one that is closest to now
        let am_time = meridiem::new_12h_time(h, m, s, Meridiem::Ante).ok_or(ParseError::Unknown)?;
        let pm_time = meridiem::new_12h_time(h, m, s, Meridiem::Post).ok_or(ParseError::Unknown)?;

        Duration::min(
            relative::duration_until_time(am_time),
            relative::duration_until_time(pm_time),
        )
    };

    log::trace!("successfully parsed time, returning {duration}");

    Ok(duration)
}

/// Tries to parse a token list as a duration with units.
fn parse_unit_tokens(tokens: &[Token]) -> Result<Duration, ParseError> {
    let mut total_duration = Duration::ZERO;
    let mut current_number = 0.0;

    for token in tokens {
        log::trace!("parsing token {token:?}");

        match token {
            Token::Number(n) => current_number = *n,
            Token::Unit(u) => total_duration += u.to_duration(current_number),
            _ => return Err(ParseError::ClashingFormats),
        }
    }

    log::trace!("parsed units so far to {total_duration}");

    // the above only adds a number to the total when a unit is encountered
    // add the trailing number if it exists, as there is no unit after
    if let Token::Number(n) = tokens.last().expect("should have at least one token") {
        log::trace!("found ending number {n}");
        let Token::Unit(unit) = tokens[tokens.len() - 2] else {
            log::warn!(
                r#"token before a the last number should always be a unit!\
                found non-unit before {n} in tokens {tokens:?}"#
            );
            return Err(ParseError::Unknown);
        };

        log::trace!("adding {n} in unit smaller than {unit:?}");

        total_duration += unit
            .smaller_unit()
            .ok_or(ParseError::SmallerThanMilli(*n))?
            .to_duration(*n);
    };

    log::trace!("successfully parsed units as {total_duration}");

    Ok(total_duration)
}
