use std::str::FromStr;

use crate::utils::parse::errors::ParseError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Meridiem {
    Ante,
    Post,
}

impl Meridiem {
    pub const AM_TOKENS: [&str; 2] = ["am", "a.m."];
    pub const PM_TOKENS: [&str; 2] = ["pm", "p.m."];
}

impl FromStr for Meridiem {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            s if Self::AM_TOKENS.contains(&s) => Self::Ante,
            s if Self::PM_TOKENS.contains(&s) => Self::Post,
            s => Err(Self::Err::InvalidUnit(s.to_string()))?,
        })
    }
}
