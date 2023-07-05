use std::fmt;

/// The error type for `parse::parse_input`.
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
    Unknown,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NaN => {
                write!(f, "Unknown number")
            }
            Self::InvalidCharacter(c) => {
                write!(f, "Invalid character \"{c}\"")
            }
            Self::InvalidNumber(n) => {
                write!(f, "Invalid number \"{n}\"")
            }
            Self::InvalidUnit(u) => {
                write!(f, "Invalid unit \"{u}\"")
            }
            Self::SmallerThanMilli(n) => {
                write!(f, "Value \"{n}\" is less than a millisecond")
            }
            Self::ClashingFormats => {
                write!(f, "Multiple formats detected")
            }
            Self::TooManySeparators => {
                write!(f, "Maximum of 2 \":\"s allowed")
            }
            Self::Empty => write!(f, "Please include a number"),
            Self::Unknown => {
                write!(f, "Invalid input")
            }
        }
    }
}

impl std::error::Error for ParseError {}
