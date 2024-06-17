mod eval;
mod lexer;
mod parser;

use std::{
    iter::{self, Peekable},
    ops::Range,
};

use dyn_clone::DynClone;
use itertools::Itertools;
use thiserror::Error;
use time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Expr(Range<usize>),
    Int(Range<usize>),
    Plus,
    Star,
    LParen,
    RParen,
}

impl Segment {
    const OP_CHARS: [char; 4] = ['+', '*', '(', ')'];

    pub fn is_empty(&self) -> bool {
        matches!(
            self,
            Self::Expr(e) | Self::Int(e) if e.is_empty()
        )
    }

    pub fn is_expr_or_int(&self) -> bool {
        matches!(self, Self::Expr(..) | Self::Int(..))
    }

    /// Whether self is any of the operator chars
    pub fn is_op(&self) -> bool {
        !self.is_expr_or_int()
    }

    /// Whether self is a binary operator, i.e. + or *
    pub fn is_bin_op(&self) -> bool {
        matches!(self, Self::Plus | Self::Star)
    }

    pub fn from_range(s: &str, range: Range<usize>) -> Self {
        match &s[range.clone()] {
            "+" => Self::Plus,
            "*" => Self::Star,
            "(" => Self::LParen,
            ")" => Self::RParen,
            s if s.chars().all(|c| c.is_ascii_digit()) => Self::Int(range),
            _ => Self::Expr(range),
        }
    }

    #[cfg(test)]
    fn to_str<'s>(&self, s: &'s str) -> &'s str {
        match self {
            Self::Expr(range) | Self::Int(range) => &s[range.clone()],
            Self::Plus => "+",
            Self::Star => "*",
            Self::LParen => "(",
            Self::RParen => ")",
        }
    }
}

fn to_rpn(input: &str) -> Result<(String, Vec<Segment>)> {
    let input = input.replace(' ', "");
    if input.is_empty() {
        return Err(Error::Empty);
    }

    // split input by operators
    let mut segments = Vec::<Segment>::new();
    let mut prev_i = 0;
    for (curr_i, curr_char) in input.char_indices() {
        if Segment::OP_CHARS.contains(&curr_char) {
            let prev_segment = Segment::from_range(&input, prev_i..curr_i);
            // character at curr_i is an operator, must be a single ascii char
            // so +1 is safe
            let curr_segment = Segment::from_range(&input, curr_i..curr_i + 1);

            // two operators in a row, e.g. "1+(2+3)"
            // don't add the empty string as an expression
            if !prev_segment.is_empty() {
                segments.push(prev_segment.clone());

                // if the input is like 3(...) or (...)4, insert a * for
                // implicit multiplication
                if (prev_segment.is_expr_or_int() && curr_segment == Segment::LParen)
                    || (prev_segment == Segment::RParen && curr_segment.is_expr_or_int())
                {
                    segments.push(Segment::Star)
                }
            }

            // push current operator
            segments.push(Segment::from_range(&input, curr_i..curr_i + 1));
            // set to next expression
            prev_i = curr_i + 1;
        }
    }

    // insert the last segment unless the input ends with an operator,
    // don't add an empty expression
    if prev_i != input.len() {
        // check for implicit multiplication
        let prev_segment = segments.last().expect("input should not be empty");
        let curr_segment = Segment::from_range(&input, prev_i..input.len());
        if prev_segment == &Segment::RParen && curr_segment.is_expr_or_int() {
            segments.push(Segment::Star)
        }
        segments.push(curr_segment);
    }

    // simple input validation: checking consecutive elements, just enough
    // to construct a syntax tree.
    // track current number of nested parens
    let mut parens: u32 = 0;
    if segments[0] == Segment::LParen {
        parens += 1;
    }

    for (prev, curr) in segments.iter().tuple_windows() {
        // TODO: use better error variants
        match curr {
            Segment::Expr(_) | Segment::Int(_) => {
                if !prev.is_op() {
                    return Err(Error::Unknown);
                }
            }
            Segment::Plus | Segment::Star => {
                if prev.is_bin_op() {
                    return Err(Error::Unknown);
                }
            }
            Segment::LParen => parens += 1,
            Segment::RParen => parens = parens.checked_sub(1).ok_or(Error::UnbalancedParens)?,
        }
    }
    if parens != 0 {
        return Err(Error::UnbalancedParens);
    }

    // convert to RPN
    // using the Shunting yard algorithm
    // https://en.wikipedia.org/wiki/Shunting_yard_algorithm
    let mut output = Vec::<Segment>::new();
    // must never contain an expr or int variant
    let mut op_stack = Vec::<Segment>::new();
    for segment in segments {
        match segment {
            Segment::Expr(_) | Segment::Int(_) => output.push(segment),
            Segment::Plus => {
                while let Some(op) = op_stack.pop_if(|op| *op != Segment::LParen) {
                    output.push(op)
                }
                op_stack.push(segment)
            }
            Segment::Star => {
                while let Some(op) =
                    op_stack.pop_if(|op| *op != Segment::LParen && *op != Segment::Plus)
                {
                    output.push(op)
                }
                op_stack.push(segment)
            }
            Segment::LParen => op_stack.push(segment),
            Segment::RParen => {
                while let Some(op) = op_stack.pop_if(|op| *op != Segment::LParen) {
                    output.push(op)
                }
                op_stack.pop().expect("parens are balanced, there must be a left paren remaining in the operator stack");
            }
        }
    }
    // push the rest of the operators onto the output
    op_stack.reverse();
    output.extend(op_stack);

    Ok((input, output))
}

trait ClonableIterator: DynClone + Iterator {}
impl<T> ClonableIterator for T where T: DynClone + Iterator {}
dyn_clone::clone_trait_object!(ClonableIterator<Item = Range<usize>>);

#[derive(Clone)]
enum ExprsOrInt {
    Exprs(Box<dyn ClonableIterator<Item = Range<usize>>>),
    Int(Range<usize>),
}

impl ExprsOrInt {
    pub fn new_exprs(exprs: impl ClonableIterator<Item = Range<usize>> + 'static) -> Self {
        Self::Exprs(Box::new(exprs))
    }
}

impl IntoIterator for ExprsOrInt {
    type Item = Range<usize>;

    type IntoIter = Box<dyn ClonableIterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ExprsOrInt::Exprs(e) => e,
            ExprsOrInt::Int(i) => Box::new(iter::once(i)),
        }
    }
}

pub struct MultiInput {
    input: String,
    ranges: Peekable<Box<dyn ClonableIterator<Item = Range<usize>>>>,
}

impl MultiInput {
    fn new(input: String, ranges: Box<dyn ClonableIterator<Item = Range<usize>>>) -> Self {
        Self {
            input,
            ranges: ranges.peekable(),
        }
    }

    pub fn next(&mut self) -> Option<&str> {
        self.ranges.next().map(|range| &self.input[range])
    }

    pub fn peek(&mut self) -> Option<&str> {
        self.ranges.peek().map(|range| &self.input[range.clone()])
    }
}

pub fn interpret_multi(input: &str) -> Result<MultiInput> {
    let (input, mut rpn) = to_rpn(input)?;
    rpn.reverse();

    let mut exprs = Vec::<ExprsOrInt>::new();
    while let Some(seg) = rpn.pop() {
        match seg {
            Segment::Expr(e) => exprs.push(ExprsOrInt::new_exprs(iter::once(e))),
            Segment::Int(i) => exprs.push(ExprsOrInt::Int(i)),
            Segment::Plus => {
                let (Some(right), Some(left)) = (exprs.pop(), exprs.pop()) else {
                    return Err(Error::Unknown);
                };
                exprs.push(ExprsOrInt::new_exprs(left.into_iter().chain(right)))
            }
            Segment::Star => {
                let (Some(right), Some(left)) = (exprs.pop(), exprs.pop()) else {
                    return Err(Error::Unknown);
                };

                // figure out which is an int and which is the expression to be multiplied
                let new_iter: Box<dyn ClonableIterator<Item = Range<usize>>> = match (left, right) {
                    (ExprsOrInt::Exprs(_), ExprsOrInt::Exprs(_)) => return Err(Error::Unknown),
                    (ExprsOrInt::Exprs(e), ExprsOrInt::Int(i))
                    | (ExprsOrInt::Int(i), ExprsOrInt::Exprs(e)) => {
                        Box::new(iter::repeat_n(e, input[i].parse().unwrap()).flatten())
                    }
                    (ExprsOrInt::Int(l), ExprsOrInt::Int(r)) => {
                        // assume that l is the time, r is the repetitions
                        Box::new(iter::repeat_n(l, input[r].parse().unwrap()))
                    }
                };

                exprs.push(ExprsOrInt::Exprs(new_iter))
            }
            Segment::LParen | Segment::RParen => unreachable!(),
        }
    }
    debug_assert_eq!(exprs.len(), 1);
    let iter = exprs.remove(0);

    Ok(MultiInput::new(input, iter.into_iter()))
}

/// Tries to parse a user inputted string as a duration.
///
/// There are 2 main formats:
/// - A duration, specified with units like "1h 30m".
///     - The units accepted are days, hours, minutes, seconds and
///       milliseconds. Several different ways of writing each are accepted
///       (e.g. "h", "hrs", "hours").
///     - If no units are given, minutes is assumed.
///     - If the string ends in a number with no unit, it is assumed to be one
///       unit smaller than the previous (e.g. "2m 30" is the same as "2m 30s").
///     - Decimals are accepted, like "3.5h".
/// - A specific time, like "5:30pm". Finds the duration until the next
///   occurrence of the specified time.
///     - If "am" or "pm" is added, the duration until the next occurrence of
///       that time is returned.
///     - If no "am" or "pm" is added, it will be interpreted as the closest one
///       (e.g. at 2pm, "3:30" is the same as "3:30pm" and "1:30" is the same
///       as "1:30am").
///     - A no-meridiem time with only the hour time can be inputted by adding
///       a ":" (e.g. "3" is interpreted as 3 minutes while "3:" is interpreted
///       as 3 am/pm, whichever is closest).
///
/// # Errors
/// Errors if the input does not match any of the above formats.
///
/// The error reason will try to be given, however it may be inconsistent
/// and change if the implementation is modified.
///
/// # Examples
/// ```rust
/// use time::{Duration, ext::NumericalDuration};
/// # use minti_ui::interpreter::interpret;
///
/// assert_eq!(interpret("3").unwrap(), 3.minutes());
/// assert_eq!(
///     interpret("3h 20m 10").unwrap(),
///     3.hours() + 20.minutes() + 10.seconds()
/// );
/// ```
pub fn interpret_single(input: &str) -> Result<Duration> {
    log::debug!("parsing input {input}");

    let groups = lexer::lex(input)?;
    let tokens = parser::parse(groups)?;
    log::trace!("successfully mapped to parsed tokens");

    eval::eval(&tokens)
}

/// The error type for `parse::parse_input`.
#[derive(Debug, PartialEq, Clone, Error)]
pub enum Error {
    #[error("Unknown number")]
    NaN,
    #[error("Invalid character \"{0}\"")]
    InvalidCharacter(char),
    #[error("Invalid number \"{0}\"")]
    InvalidNumber(String),
    #[error("Invalid unit \"{0}\"")]
    InvalidUnit(String),
    #[error("Value \"{0}\" is less than a millisecond")]
    SmallerThanMilli(f64),
    #[error("Multiple formats detected")]
    ClashingFormats,
    #[error("Maximum of 2 \":\"s allowed")]
    TooManySeparators,
    #[error("No input provided")]
    Empty,
    #[error("Invalid input")]
    Unknown,
    #[error("Unbalanced parentheses")]
    UnbalancedParens,
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::ext::NumericalDuration;

    mod multi {
        use itertools::Itertools;

        use super::{interpret_multi, to_rpn};

        fn print_input_rpn(input: &str) {
            let Ok((stripped, segments)) =
                to_rpn(input).inspect_err(|e| eprintln!("{input} => {e}"))
            else {
                return;
            };

            let res = segments
                .into_iter()
                .map(|seg| seg.to_str(&stripped))
                .collect_vec();
            println!("{stripped} => {res:?}");
        }

        #[test]
        fn multi_input() {
            // validate input: ei = expression or int
            // - must have balanced parentheses
            // - expressions/ints must be separated by operators (i.e. no `expr int`)
            // - plus must be  ei+ei    | )+ei     | ei+(  | )+(
            // - star must be  expr*int | int*expr | )*int | int*( | ei* | )*
            // - paren must be int(ei   | ei)int   | +(ei  | *(ei  |

            print_input_rpn("3*2m");
            print_input_rpn("1+1");
            print_input_rpn("2+(3*2)");
            print_input_rpn("(15+45)*");
            print_input_rpn("3(40h)");
            print_input_rpn("(1+3)4");
            print_input_rpn("4*(1+3)");
            print_input_rpn("7+8+4*(1+3+2)+5+6");
            print_input_rpn("7+8+(1+3+2)*4+5+6*2");
        }

        #[test]
        fn actual_multi_input() {
            let mut iter = interpret_multi("7h+8+(1+3)*2+5s+6*2").unwrap();
            assert_eq!(iter.next(), Some("7h"));
            assert_eq!(iter.next(), Some("8"));
            assert_eq!(iter.next(), Some("1"));
            assert_eq!(iter.next(), Some("3"));
            assert_eq!(iter.next(), Some("1"));
            assert_eq!(iter.next(), Some("3"));
            assert_eq!(iter.next(), Some("5s"));
            assert_eq!(iter.next(), Some("6"));
            assert_eq!(iter.next(), Some("6"));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn plain_int_as_mins() {
        assert_eq!(interpret_single("23").unwrap(), 23.minutes());
        assert_eq!(interpret_single("938").unwrap(), 938.minutes());
        assert_eq!(interpret_single("0").unwrap(), 0.minutes());
    }

    mod units {
        use super::*;

        #[test]
        fn single_units() {
            assert_eq!(interpret_single("3h").unwrap(), 3.hours());
            assert_eq!(interpret_single("10 h").unwrap(), 10.hours());
            assert_eq!(interpret_single("1.61 h").unwrap(), 1.61.hours());
            assert_eq!(interpret_single("2 hours").unwrap(), 2.hours());

            assert_eq!(interpret_single("3m").unwrap(), 3.minutes());
            assert_eq!(interpret_single("49ms").unwrap(), 49.milliseconds());
        }

        #[test]
        fn multiple_units() {
            assert_eq!(interpret_single("3h21m").unwrap(), 3.hours() + 21.minutes());

            assert_eq!(
                interpret_single("8d 23h 12m 5s 91ms").unwrap(),
                8.days() + 23.hours() + 12.minutes() + 5.seconds() + 91.milliseconds()
            )
        }

        #[test]
        fn trailing_number() {
            assert_eq!(interpret_single("3h4").unwrap(), 3.hours() + 4.minutes());

            assert_eq!(
                interpret_single("3d 23h 12.3m 2").unwrap(),
                3.days() + 23.hours() + 12.3.minutes() + 2.seconds()
            )
        }
    }

    mod times {
        use crate::time::relative::duration_until_time;
        use time::Time;

        use super::*;

        #[test]
        fn specific_12h_time() {
            assert_eq!(
                interpret_single("3pm").unwrap().whole_seconds(),
                duration_until_time(Time::from_hms(3 + 12, 0, 0).unwrap()).whole_seconds()
            );

            assert_eq!(
                interpret_single("3:12pm").unwrap().whole_seconds(),
                duration_until_time(Time::from_hms(3 + 12, 12, 0).unwrap()).whole_seconds()
            );

            assert_eq!(
                interpret_single("5:12:30 am").unwrap().whole_seconds(),
                duration_until_time(Time::from_hms(5, 12, 30).unwrap()).whole_seconds()
            );
        }
    }

    mod errors {
        use super::*;
        fn all_errors(values: &[&str]) {
            for value in values {
                assert!(
                    interpret_single(value).is_err(),
                    "{value} should have been an Err."
                )
            }
        }

        #[test]
        fn raises_error() {
            all_errors(&[
                "3.24x",
                "abc",
                "3:5:6:2:1",
                "",
                "h",
                "10s 300ms 10",
                "13:0:0am",
                "3pm 10",
            ])
        }
    }
}
