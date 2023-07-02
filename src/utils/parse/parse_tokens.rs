use time::{Duration, ext::NumericalDuration};

use crate::utils::{
    time::{
        meridiem::{self, Meridiem},
        relative,
    },
};

use super::{
    errors::ParseError,
    structs::{Token, TokensFormat},
};

pub(super) fn parse_tokens(tokens: &[Token]) -> Result<Duration, ParseError> {
    let format = get_tokens_format(tokens);

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
        return Err(ParseError::Empty);
    };
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
        // There should be a maximum of one am/pm
        // This would not run if it was set on the last iteration
        if meridiem.is_some() {
            return Err(ParseError::Unknown);
        };

        #[expect(clippy::match_wildcard_for_single_variants)]
        match token {
            Token::Separator => {
                current_unit += 1;
                // check needs to be here to avoid an index error
                if current_unit > 2 {
                    return Err(ParseError::TooManySeparators);
                }
            }
            // only allow times with integers
            Token::Number(n) if n.fract() == 0.0 => time_sections[current_unit] = *n as u8,
            Token::Number(n) => return Err(ParseError::InvalidNumber(n.to_string())),
            Token::Meridiem(m) => meridiem = Some(*m),
            _ => return Err(ParseError::ClashingFormats),
        }
    }

    let [h, m, s] = time_sections;

    let duration = if let Some(meri) = meridiem {
        relative::duration_until_time(
            meridiem::new_12h_time(h, m, s, meri).ok_or(ParseError::Unknown)?,
        )
    } else {
        // find the one that is closest to now
        let am_time = meridiem::new_12h_time(h, m, s, Meridiem::Ante).ok_or(ParseError::Unknown)?;
        let pm_time = meridiem::new_12h_time(h, m, s, Meridiem::Post).ok_or(ParseError::Unknown)?;

        Duration::min(
            relative::duration_until_time(am_time),
            relative::duration_until_time(pm_time),
        )
    };

    Ok(duration)
}

/// Tries to parse a token list as a duration with units.
fn parse_unit_tokens(tokens: &[Token]) -> Result<Duration, ParseError> {
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
