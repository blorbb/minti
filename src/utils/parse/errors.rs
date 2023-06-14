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
