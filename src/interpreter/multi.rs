use dyn_clone::DynClone;
use itertools::Itertools;
use std::{
    iter::{self, Peekable},
    ops::Range,
};

use super::{Error, Result};

pub fn interpret_multi(input: &str) -> Result<MultiInput> {
    let (input, mut rpn) = to_rpn(input)?;
    rpn.reverse();

    let mut exprs = Vec::<ExprsOrInt>::new();
    while let Some(seg) = rpn.pop() {
        match seg {
            Segment::Expr(e) => exprs.push(ExprsOrInt::new_exprs(iter::once(e))),
            Segment::Int(i) => exprs.push(ExprsOrInt::Int(i)),
            Segment::Inf => todo!(),
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
            Segment::LParen | Segment::RParen | Segment::Eof => unreachable!(),
        }
    }
    debug_assert_eq!(exprs.len(), 1);
    let iter = exprs.remove(0);

    Ok(MultiInput::new(input, iter.into_iter()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Expr(Range<usize>),
    Int(Range<usize>),
    Plus,
    Star,
    Inf,
    LParen,
    RParen,
    Eof,
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
        matches!(self, Self::Expr(..) | Self::Int(..) | Self::Inf)
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
            "\0" => Self::Eof,
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
            Self::Inf => "inf",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::Eof => "eof",
        }
    }
}

fn to_rpn(input: &str) -> Result<(String, Vec<Segment>)> {
    let mut input = input.replace(' ', "");
    if input.is_empty() {
        return Err(Error::Empty);
    }

    // extra character makes it easier to correctly parse the end
    input.push('\0');

    // split input by operators
    let mut segments = Vec::<Segment>::new();
    let mut prev_i = 0;
    for (curr_i, curr_char) in input.char_indices() {
        if Segment::OP_CHARS.contains(&curr_char) || curr_char == '\0' {
            let prev_segment = Segment::from_range(&input, prev_i..curr_i);
            // character at curr_i is an operator, must be a single ascii char
            // so +1 is safe
            let curr_segment = Segment::from_range(&input, curr_i..curr_i + 1);

            // two operators in a row, e.g. "1+(2+3)"
            // don't add the empty string as an expression
            if !prev_segment.is_empty() {
                segments.push(prev_segment.clone());
            }

            // push current operator
            segments.push(curr_segment);
            // set to next expression
            prev_i = curr_i + 1;
        }
    }

    // insert the implicit multiplication between bracket and int/expr
    let segments = segments
        .into_iter()
        .tuple_windows()
        .flat_map(|(curr, next)| {
            if curr == Segment::RParen && next.is_expr_or_int()
                || curr.is_expr_or_int() && next == Segment::LParen
            {
                itertools::Either::Left([curr, Segment::Star].into_iter())
            } else {
                itertools::Either::Right([curr].into_iter())
            }
        })
        // curr never reaches the last element, so re-insert the EOF
        .chain(iter::once(Segment::Eof))
        .collect_vec();

    // input validation: checking consecutive elements.
    // also converting stars to postfix stars if possible.
    let mut parens: u32 = 0;

    let segments = segments
        .into_iter()
        .tuple_windows()
        .map(|(curr, next)| match curr {
            // no consecutive exprs/ints (should not be possible to
            // construct this anyways)
            Segment::Expr(_) | Segment::Int(_) if next.is_expr_or_int() => Err(Error::Unknown),
            // segment after plus should be expr/int/(
            Segment::Plus if !(next.is_expr_or_int() || next == Segment::LParen) => {
                Err(Error::Unknown)
            }
            // figure out if its multiplying two things, or the postfix star for infinite loop
            Segment::Star => {
                if next.is_expr_or_int() || next == Segment::LParen {
                    // binary multiply
                    Ok(itertools::Either::Left(iter::once(Segment::Star)))
                } else {
                    // postfix multiply
                    Ok(itertools::Either::Right(
                        [Segment::Star, Segment::Inf].into_iter(),
                    ))
                }
            }
            Segment::LParen if next.is_bin_op() => Err(Error::Unknown),
            Segment::LParen => {
                parens += 1;
                Ok(itertools::Either::Left(iter::once(curr)))
            }
            Segment::RParen => {
                parens = parens.checked_sub(1).ok_or(Error::UnbalancedParens)?;
                Ok(itertools::Either::Left(iter::once(curr)))
            }
            Segment::Eof => unreachable!("curr can never be eof"),
            Segment::Inf => unreachable!("infinities have not been constructed yet"),
            _ => Ok(itertools::Either::Left(iter::once(curr))),
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect_vec();

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
            Segment::Expr(_) | Segment::Int(_) | Segment::Inf => output.push(segment),
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
            Segment::Eof => unreachable!("eof should have been removed"),
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

    #[cfg(test)]
    fn collect(&mut self) -> Vec<&str> {
        let mut vec = Vec::new();
        // needs to be manual because
        while let Some(range) = self.ranges.next() {
            vec.push(&self.input[range])
        }
        vec
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::interpreter::Error;

    use super::interpret_multi;

    use super::to_rpn;

    #[track_caller]
    fn validate_rpn(input: &str, expected: &[&str]) {
        let (input, segments) = to_rpn(input)
            .unwrap_or_else(|e| panic!("input could not be converted to rpn: got error {e}"));
        let rpn = segments
            .into_iter()
            .map(|seg| seg.to_str(&input))
            .collect_vec();
        assert_eq!(rpn, expected)
    }

    #[test]
    fn convert_to_rpn() {
        // validate input: ei = expression or int
        // - must have balanced parentheses
        // - expressions/ints must be separated by operators (i.e. no `expr int`)
        // - plus must be  ei+ei    | )+ei     | ei+(  | )+(
        // - star must be  expr*int | int*expr | )*int | int*( | ei* | )*
        // - paren must be int(ei   | ei)int   | +(ei  | *(ei  |

        validate_rpn("3*2m", &["3", "2m", "*"]);
        validate_rpn("1+1", &["1", "1", "+"]);
        validate_rpn("1+(3*2)", &["1", "3", "2", "*", "+"]);
        validate_rpn("(15+45m)*", &["15", "45m", "+", "*inf"]);
        validate_rpn("3(40h)", &["3", "40h", "*"]);
        validate_rpn("(1+3)4", &["1", "3", "+", "4", "*"]);
        validate_rpn("4*(1+3)", &["4", "1", "3", "+", "*"]);
        validate_rpn(
            "7+8+(1+3+2)*4+5+6*2",
            &[
                "7", "8", "+", "1", "3", "+", "2", "+", "4", "*", "+", "5", "+", "6", "2", "*", "+",
            ],
        );
        // TODO: this needs to be figured out, disambiguate as unary *
        validate_rpn("2m+3*", &["2m", "3", "*inf", "+"])
    }

    #[test]
    fn multi_with_next() {
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

    #[track_caller]
    fn validate_multi(input: &str, expected: &[&str]) {
        let mut iter = interpret_multi(input).unwrap();
        assert_eq!(iter.collect(), expected);
    }

    #[test]
    fn more_multi_input() {
        validate_multi("(10m+45)*3", &["10m", "45", "10m", "45", "10m", "45"]);
        validate_multi("(10m+45)3", &["10m", "45", "10m", "45", "10m", "45"]);
        validate_multi("3(10m+45)", &["10m", "45", "10m", "45", "10m", "45"]);
        validate_multi("3*10m + 45", &["10m", "10m", "10m", "45"]);
        validate_multi("3*2m", &["2m", "2m", "2m"]);
        validate_multi("1+1", &["1", "1"]);
        validate_multi("1+(3*2)", &["1", "3", "3"]);
        validate_multi("(15+45m)", &["15", "45m"]);
        validate_multi("3(40h)", &["40h", "40h", "40h"]);
        validate_multi("(1+3)4", &["1", "3", "1", "3", "1", "3", "1", "3"]);
        validate_multi("4*(1+3)", &["1", "3", "1", "3", "1", "3", "1", "3"]);
        assert_eq!(interpret_multi("(1+3)*").err(), Some(Error::Unknown));
        assert_eq!(interpret_multi("2m+3*").err(), Some(Error::Unknown));
    }
}
