use std::fmt;

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
            ParseError::NaN => {
                write!(f, "Unknown number")
            }
            ParseError::InvalidCharacter(c) => {
                write!(f, "Invalid character \"{c}\"")
            }
            ParseError::InvalidNumber(n) => {
                write!(f, "Invalid number \"{n}\"")
            }
            ParseError::InvalidUnit(u) => {
                write!(f, "Invalid unit \"{u}\"")
            }
            ParseError::SmallerThanMilli(n) => {
                write!(f, "Value \"{n}\" is less than a millisecond")
            }
            ParseError::ClashingFormats => {
                write!(f, "Multiple formats detected")
            }
            ParseError::TooManySeparators => {
                write!(f, "Maximum of 2 \":\"s allowed")
            }
            ParseError::Empty => write!(f, "Please include a number"),
            ParseError::Unknown => {
                write!(f, "Invalid input")
            }
        }
    }
}
