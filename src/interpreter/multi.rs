use az::SaturatingAs;
use dyn_clone::DynClone;
use itertools::{Either, Itertools};
use std::{
    fmt::{self, Write},
    iter::{self, Peekable},
    sync::Arc,
};

use super::{Error, Result};

// pratt parser based on
// https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html

pub fn interpret_multi(input: &str) -> Result<InputIter> {
    let expr = parse(input)?;
    Ok(InputIter::from(eval(expr)?))
}

pub struct InputIter {
    iter: Peekable<Box<dyn ClonableIterator<Item = Arc<str>>>>,
}

impl InputIter {
    pub fn empty() -> Self {
        let iter: Box<dyn ClonableIterator<Item = Arc<str>>> = Box::new(iter::empty()); 
        Self {
            iter: iter.peekable()
        }
    }

    pub fn next(&mut self) -> Option<Arc<str>> {
        self.iter.next()
    }

    pub fn peek(&mut self) -> Option<Arc<str>> {
        self.iter.peek().cloned()
    }
}

impl From<DurationsOrInt> for InputIter {
    fn from(value: DurationsOrInt) -> Self {
        let iter = match value {
            DurationsOrInt::Durations(durations) => durations,
            DurationsOrInt::Int(int) => Box::new(iter::once(Arc::from(int.to_string()))),
        };
        Self {
            iter: iter.peekable(),
        }
    }
}

fn eval(sexpr: SExpr) -> Result<DurationsOrInt> {
    match sexpr {
        SExpr::Atom(value) => Ok(DurationsOrInt::from(value)),
        SExpr::Cons(op, exprs) => {
            let [left, right] = *exprs;
            let (left, right) = (eval(left)?, eval(right)?);
            match op {
                Op::Add => Ok(left.join(right)),
                Op::Mul => match (left, right) {
                    (DurationsOrInt::Durations(_), DurationsOrInt::Durations(_)) => {
                        Err(Error::MulDurations)
                    }
                    (DurationsOrInt::Durations(d), DurationsOrInt::Int(int))
                    | (DurationsOrInt::Int(int), DurationsOrInt::Durations(d)) => {
                        Ok(DurationsOrInt::durations(
                            iter::repeat_n(d, int.saturating_as::<usize>()).flatten(),
                        ))
                    }
                    (DurationsOrInt::Int(l), DurationsOrInt::Int(r)) => {
                        let l = Arc::from(l.to_string());
                        Ok(DurationsOrInt::durations(iter::repeat_n(
                            l,
                            r.saturating_as::<usize>(),
                        )))
                    }
                },
                Op::LParen | Op::RParen => unreachable!(),
            }
        }
    }
}

fn parse(input: &str) -> Result<SExpr> {
    let mut lexer = Lexer::new(input);

    // check that every Value::Duration is a valid duration expression too
    for token in &lexer.tokens {
        match token {
            Token::Value(Value::Duration(str)) => _ = super::interpret_single(&str)?,
            _ => (),
        }
    }

    let result = expr_bp(&mut lexer, 0)?;
    if lexer.peek() != Token::Eof {
        Err(Error::UnbalancedParens)
    } else {
        Ok(result)
    }
}

fn expr_bp(lexer: &mut Lexer, min_bp: u8) -> Result<SExpr> {
    let mut lhs = match lexer.next() {
        Token::Op(Op::LParen) => {
            let lhs = expr_bp(lexer, 0)?;
            let next = lexer.next();
            if next != Token::RPAREN {
                return Err(Error::UnbalancedParens);
            }
            lhs
        }
        Token::Value(val) => SExpr::Atom(val),
        Token::Op(op) => return Err(Error::InvalidOp(op.to_string())),
        Token::Eof => return Err(Error::Empty),
    };

    loop {
        let op = match lexer.peek() {
            Token::Eof => break,
            Token::Op(op) => op,
            t => return Err(Error::InvalidValue(t.to_string())),
        };

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }

            lexer.next();
            let rhs = match expr_bp(lexer, r_bp) {
                Ok(rhs) => rhs,
                Err(Error::Empty) => return Err(Error::InvalidOp(op.to_string())),
                Err(e) => return Err(e),
            };
            lhs = SExpr::Cons(op, Box::new([lhs, rhs]));

            continue;
        }

        break;
    }

    Ok(lhs)
}

fn infix_binding_power(op: Op) -> Option<(u8, u8)> {
    Some(match op {
        Op::Add => (1, 2),
        Op::Mul => (3, 4),
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Value(Value),
    Op(Op),
    Eof,
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        match value {
            "+" => Self::Op(Op::Add),
            "*" => Self::Op(Op::Mul),
            "(" => Self::Op(Op::LParen),
            ")" => Self::Op(Op::RParen),
            "\0" => Self::Eof,
            string => Self::Value(match string.parse::<u64>() {
                Ok(int) => Value::Int(int),
                Err(_) => Value::Duration(Arc::from(string)),
            }),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(v) => write!(f, "{v}"),
            Self::Op(op) => write!(f, "{op}"),
            Self::Eof => write!(f, "eof"),
        }
    }
}

impl Token {
    const MUL: Self = Token::Op(Op::Mul);
    const LPAREN: Self = Token::Op(Op::LParen);
    const RPAREN: Self = Token::Op(Op::RParen);

    /// Returns `true` if the token is [`Value`].
    ///
    /// [`Value`]: Token::Value
    #[must_use]
    fn is_value(&self) -> bool {
        matches!(self, Self::Value(..))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Duration(Arc<str>),
    Int(u64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duration(e) => write!(f, "{e}"),
            Self::Int(i) => write!(f, "{i}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Mul,
    LParen,
    RParen,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => f.write_char('+'),
            Op::Mul => f.write_char('*'),
            Op::LParen => f.write_char('('),
            Op::RParen => f.write_char(')'),
        }
    }
}

struct Lexer {
    /// Stored in reverse order
    tokens: Vec<Token>,
}

impl fmt::Debug for Lexer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        f.write_str(&self.tokens.iter().rev().join(" "))?;
        f.write_char('"')
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        // end with eof, makes some stuff easier
        let mut tokens = format!("{input}\0")
            .split_inclusive(&['+', '*', '(', ')', '\0'])
            .flat_map(|segment| <[&str; 2]>::from(segment.split_at(segment.len() - 1)))
            .map(|s| s.trim())
            .filter_map(|s| (!s.is_empty()).then(|| Token::from(s)))
            // normalize input
            .tuple_windows()
            .flat_map(|(curr, next)| {
                // insert implicit multiply
                if (curr == Token::RPAREN && next.is_value())
                    || (curr.is_value() && next == Token::LPAREN)
                {
                    Either::Left([curr, Token::Op(Op::Mul)].into_iter())
                } else if curr == Token::MUL && !(next == Token::LPAREN || next.is_value()) {
                    // postfix multiply
                    Either::Left([curr, Token::Value(Value::Int(u64::MAX))].into_iter())
                } else {
                    Either::Right(iter::once(curr))
                }
            })
            .collect_vec();

        tokens.reverse();
        Self { tokens }
    }

    pub fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }

    pub fn peek(&mut self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token::Eof)
    }
}

/// An S-expression
#[derive(Debug)]
enum SExpr {
    Atom(Value),
    Cons(Op, Box<[SExpr; 2]>),
}

impl fmt::Display for SExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SExpr::Atom(t) => write!(f, "{t}"),
            SExpr::Cons(head, rest) => {
                write!(f, "({head}")?;
                for s in rest.iter() {
                    write!(f, " {s}")?
                }
                write!(f, ")")
            }
        }
    }
}

trait ClonableIterator: DynClone + Iterator {}
impl<T> ClonableIterator for T where T: DynClone + Iterator {}
dyn_clone::clone_trait_object!(<T> ClonableIterator<Item = T>);

/// An evaluated duration expression.
///
/// Any expressions that may still be interpreted as an integer will be the [`Int`]
/// variant. Only when it must be interpreted as a duration will the integer become
/// a [`Durations`] variant. All other values that cannot be interpreted as an
/// integer will also be the [`Durations`] variant.
///
/// [`Durations`]: DurationsOrInt::Durations
/// [`Int`]: DurationsOrInt::Int
#[derive(Clone)]
enum DurationsOrInt {
    Durations(Box<dyn ClonableIterator<Item = Arc<str>>>),
    Int(u64),
}

impl DurationsOrInt {
    pub fn durations(
        it: impl IntoIterator<IntoIter: ClonableIterator, Item = Arc<str>> + 'static,
    ) -> Self {
        Self::Durations(Box::new(it.into_iter()))
    }

    pub fn into_durations(self) -> Box<dyn ClonableIterator<Item = Arc<str>>> {
        match self {
            DurationsOrInt::Durations(d) => d,
            DurationsOrInt::Int(int) => Box::new(iter::once(Arc::from(int.to_string()))),
        }
    }

    pub fn join(self, other: Self) -> Self {
        Self::durations(self.into_durations().chain(other.into_durations()))
    }
}

impl From<Value> for DurationsOrInt {
    fn from(value: Value) -> Self {
        match value {
            Value::Duration(d) => DurationsOrInt::durations(iter::once(d)),
            Value::Int(int) => DurationsOrInt::Int(int),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        assert_eq!(parse("1 + 2 * 3").unwrap().to_string(), "(+ 1 (* 2 3))");
        assert_eq!(parse("1").unwrap().to_string(), "1");
        assert_eq!(
            parse("a + b * c * d + e").unwrap().to_string(),
            "(+ (+ a (* (* b c) d)) e)"
        );
        assert_eq!(
            parse("a + b*").unwrap().to_string(),
            format!("(+ a (* b {}))", u64::MAX)
        );
        assert_eq!(parse("2 + (3 + 4)").unwrap().to_string(), "(+ 2 (+ 3 4))");
        assert_eq!(
            parse("2h + (15m + 45)*3").unwrap().to_string(),
            "(+ 2h (* (+ 15m 45) 3))"
        );
        assert_eq!(
            parse("2h + 3*(15m + 45)").unwrap().to_string(),
            "(+ 2h (* 3 (+ 15m 45)))"
        );
        assert_eq!(
            parse("2h + 3(15m + 45)").unwrap().to_string(),
            "(+ 2h (* 3 (+ 15m 45)))"
        );
        assert_eq!(
            parse("2h + (15m + 45)*").unwrap().to_string(),
            format!("(+ 2h (* (+ 15m 45) {}))", u64::MAX)
        );
        assert_eq!(
            parse("2h+2(15+45m)2+3h").unwrap().to_string(),
            "(+ (+ 2h (* (* 2 (+ 15 45m)) 2)) 3h)"
        );
        assert_eq!(
            parse("(15*2m)*3+14d").unwrap().to_string(),
            "(+ (* (* 15 2m) 3) 14d)"
        );
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(parse("1h)").err(), Some(Error::UnbalancedParens));
        assert_eq!(parse("(1h").err(), Some(Error::UnbalancedParens));
        assert_eq!(parse("+30d").err(), Some(Error::InvalidOp("+".to_string())));
        assert_eq!(parse("30d+").err(), Some(Error::InvalidOp("+".to_string())));
        assert_eq!(
            parse("12m+34d ++ 2h").err(),
            Some(Error::InvalidOp("+".to_string()))
        );
        assert_eq!(
            parse("12m+34d +* 2h").err(),
            Some(Error::InvalidOp("*".to_string()))
        );
        assert_eq!(
            parse("1d+20(2+)").err(),
            Some(Error::InvalidOp(")".to_string()))
        );
    }

    impl InputIter {
        fn collect(self) -> Vec<String> {
            self.iter.map(|str| str.to_string()).collect_vec()
        }
    }

    #[test]
    fn eval_durations() {
        assert_eq!(
            interpret_multi("1 + 2 * 3").unwrap().collect(),
            ["1", "2", "2", "2"]
        );
        assert_eq!(interpret_multi("1").unwrap().collect(), ["1"]);
        assert_eq!(
            interpret_multi("2 + (3 + 4)").unwrap().collect(),
            ["2", "3", "4"]
        );
        assert_eq!(
            interpret_multi("2h + (15m + 45)*3").unwrap().collect(),
            ["2h", "15m", "45", "15m", "45", "15m", "45"]
        );
        assert_eq!(
            interpret_multi("2h + 3*(15m + 45)").unwrap().collect(),
            ["2h", "15m", "45", "15m", "45", "15m", "45"]
        );
        assert_eq!(
            interpret_multi("2h + 3(15m + 45)").unwrap().collect(),
            ["2h", "15m", "45", "15m", "45", "15m", "45"]
        );
        assert_eq!(
            interpret_multi("2h+2(15+45m)2+3h").unwrap().collect(),
            ["2h", "15", "45m", "15", "45m", "15", "45m", "15", "45m", "3h"]
        );
        assert_eq!(
            interpret_multi("(3*2m)*2+14d").unwrap().collect(),
            ["2m", "2m", "2m", "2m", "2m", "2m", "14d"]
        );

        let mut inps = interpret_multi("2h + 1h*").unwrap();
        assert_eq!(inps.next(), Some(Arc::from("2h")));
        assert_eq!(inps.next(), Some(Arc::from("1h")));
        assert_eq!(inps.next(), Some(Arc::from("1h")));
        assert_eq!(inps.next(), Some(Arc::from("1h")));
        assert_eq!(inps.next(), Some(Arc::from("1h")));

        let mut inps = interpret_multi("2h + (15m + 45)*").unwrap();
        assert_eq!(inps.next(), Some(Arc::from("2h")));
        assert_eq!(inps.next(), Some(Arc::from("15m")));
        assert_eq!(inps.next(), Some(Arc::from("45")));
        assert_eq!(inps.next(), Some(Arc::from("15m")));
        assert_eq!(inps.next(), Some(Arc::from("45")));
    }

    #[test]
    fn invalid_durations() {
        let inps = interpret_multi("2h + 4a + 3d");
        assert_eq!(inps.err(), Some(Error::InvalidUnit("a".to_string())));
        let inps = interpret_multi("(3h + 4 + (2ms 3)*2)*2");
        assert_eq!(inps.err(), Some(Error::SmallerThanMilli(3.0)));

        let inps = interpret_multi("3h * 2am");
        assert_eq!(inps.err(), Some(Error::MulDurations));
    }
}
