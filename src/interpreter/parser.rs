use crate::time::{meridiem::Meridiem, units::TimeUnit};

use super::{
    lexer::{Group, GroupKind},
    Error, Result,
};

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

impl TryFrom<Group> for Token {
    type Error = Error;

    fn try_from(value: Group) -> Result<Self> {
        let token = value.variant;
        let string = value.string;
        Ok(match token {
            GroupKind::Number => {
                // TODO figure out how to remove the clone
                let num = string
                    .parse::<f64>()
                    .map_err(|_| Error::InvalidNumber(string.clone()))?;

                if num.is_nan() || num.is_infinite() {
                    return Err(Error::InvalidNumber(string));
                }
                Self::Number(num)
            }
            GroupKind::Text => {
                if let Ok(n) = string.parse::<TimeUnit>() {
                    Self::Unit(n)
                } else {
                    Self::Meridiem(string.parse::<Meridiem>()?)
                }
            }
            GroupKind::Separator => Self::Separator,
        })
    }
}

pub fn parse(groups: Vec<Group>) -> Result<Vec<Token>> {
    groups.into_iter().map(Token::try_from).collect()
}
