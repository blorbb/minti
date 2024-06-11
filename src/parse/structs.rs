use crate::{time::meridiem::Meridiem, time::units::TimeUnit};

use super::ParseError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum UnparsedTokenType {
    Number,
    Text,
    Separator,
}

impl TryFrom<char> for UnparsedTokenType {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value.is_ascii_alphabetic() {
            Ok(Self::Text)
        } else if value.is_ascii_digit() || value == '.' {
            Ok(Self::Number)
        } else if value == ':' {
            Ok(Self::Separator)
        } else {
            Err(ParseError::InvalidCharacter(value))
        }
    }
}

/// A string that has one 'type' of characters.
///
/// The three variants are:
/// - `Text` if all characters are letters.
/// - `Number` if all characters are digits or ".".
/// - `Separator` if the string is ":".
#[derive(Debug, PartialEq, Eq)]
pub(super) struct UnparsedToken {
    pub variant: UnparsedTokenType,
    pub string: String,
}

#[derive(Debug, PartialEq, Eq)]
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

/// A valid token value.
///
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
                Self::Number(num)
            }
            UnparsedTokenType::Text => {
                if let Ok(n) = string.parse::<TimeUnit>() {
                    Self::Unit(n)
                } else {
                    Self::Meridiem(string.parse::<Meridiem>()?)
                }
            }
            UnparsedTokenType::Separator => Self::Separator,
        })
    }
}
