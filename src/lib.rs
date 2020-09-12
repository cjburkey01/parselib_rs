use std::error::Error;
use std::fmt::{Display, Formatter};

/// Default implementations for a few different types of parsers.
pub mod parsers;

/// Parser testing utilities.
#[cfg(test)]
mod tests;

/// A structure that contains all of the data for the current location and data for the parsing run.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ParserState {
    input: String,
    index: usize,
}

impl ParserState {
    pub fn new_offset(input: &str, index: usize) -> Self {
        Self {
            input: String::from(input),
            index,
        }
    }

    pub fn new(input: &str) -> Self {
        Self::new_offset(input, 0)
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn char(&self, offset: usize) -> Option<(Self, char)> {
        if offset >= self.input.len() {
            None
        } else {
            Some((
                Self {
                    input: String::from(&self.input[(offset + 1)..]),
                    index: self.index + offset + 1,
                },
                self.input.chars().take(offset + 1).last().unwrap(),
            ))
        }
    }

    pub fn chars(&self, count: usize) -> Option<Vec<char>> {
        if self.input.len() >= count {
            Some(self.input.chars().take(count).collect())
        } else {
            None
        }
    }
}

/// The type returned by parsers containing either the output and the new parser state or an error with more
/// information.
pub type ParseResult<ErrorType, OutputType> = Result<(OutputType, ParserState), ErrorType>;

/// Represents a parser that will take the current parser state and try to transform it.
pub trait Parser<OutputType, ErrorType> {
    /// Try to parse a piece of the input and return a parser result based on whether that is successful.
    fn parse(&self, parser_state: ParserState) -> ParseResult<ErrorType, OutputType>;
}

/// An enum of possible error types for the default provided parsers.
#[derive(Debug)]
pub enum ParseError {
    /// TBD.
    Unknown,

    /// The parser received an input that it wasn't expecting.
    Unexpected {
        expected: Option<String>,
        found: Option<String>,
    },

    /// The parser didn't receive the number of elements that were expected.
    WrongCount {
        min: usize,
        max: usize,
        found: usize,
    },
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => writeln!(f, "unknown parsing error"),
            Self::Unexpected { expected, found } => writeln!(
                f,
                "expected {} found {}",
                expected.as_ref().map_or("nothing", |expected| expected),
                found.as_ref().map_or("nothing", |found| found,)
            ),
            Self::WrongCount { min, max, found } => writeln!(
                f,
                "expected {} elements but found {}",
                if min == max {
                    min.to_string()
                } else {
                    format!("{}-{}", min, max)
                },
                found
            ),
        }
    }
}

impl Error for ParseError {}
