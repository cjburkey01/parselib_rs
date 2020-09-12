use crate::parsers::utf8::{ParseAnd, ParseChar, ParseCount};
use crate::Parser;

/// Parsers that specifically make use of the `char` type and can be used to parse strings.
pub mod utf8 {
    use super::super::*;
    use std::marker::PhantomData;

    /// Parses a single character and optionally checks whether it is within a provided range.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct ParseChar {
        /// The inclusive minimum bound of the character this parser is looking for, or `None` if it should accept
        /// characters from `0`.
        pub start: Option<char>,

        /// The inclusive maximum bound of the character this parser is looking for, or `None` if it should accept
        /// characters below the maximum `char`.
        pub end: Option<char>,
    }

    impl ParseChar {
        /// Create a character parser that will match characters between the inclusive range `start..end`.
        pub fn from_range(start: char, end: char) -> Self {
            Self {
                start: Some(start),
                end: Some(end),
            }
        }

        /// Create a character parser that will match characters above and including the `start`.
        pub fn from_start(start: char) -> Self {
            Self {
                start: Some(start),
                end: None,
            }
        }

        /// Create a character parser that will match characters below and including the `end`.
        pub fn from_end(end: char) -> Self {
            Self {
                start: None,
                end: Some(end),
            }
        }

        /// Create a character parser that will match only the provided character.
        pub fn from_char(c: char) -> Self {
            Self {
                start: Some(c),
                end: Some(c),
            }
        }

        /// Create a character parser that will match any character.
        pub fn from_any() -> Self {
            Self {
                start: None,
                end: None,
            }
        }
    }

    impl Parser<char, ParseError> for ParseChar {
        fn parse(&self, parser_state: ParserState) -> ParseResult<ParseError, char> {
            // Use a function to make this method neater when building an error.
            fn expected_str_from_char_range(start: Option<char>, end: Option<char>) -> String {
                format!(
                    "{}..{}",
                    if let Some(start) = start { start } else { '\0' },
                    if let Some(end) = end { end } else { '\0' }
                )
            }

            // Get the first character in the input
            if let Some((new_state, char_at)) = parser_state.char(0) {
                // Check if the character is larger than or at the minimum
                // bound, if there is a minimum bound.
                let above_start = self.start.map_or_else(|| true, |start| char_at >= start);

                // Check if the character is smaller than or at the maximum
                // bound, if there is a maximum bound.
                let below_end = self.end.map_or_else(|| true, |end| char_at <= end);

                // Check if the character is within the bounds.
                if above_start && below_end {
                    // If so, return the character and the parser state with
                    // its index incremented.
                    Ok((char_at, new_state))
                } else {
                    // Otherwise, the character found isn't within the provided range.
                    Err(ParseError::Unexpected {
                        expected: Some(expected_str_from_char_range(self.start, self.end)),
                        found: Some(char_at.to_string()),
                    })
                }
            } else {
                // There were no more characters to take from the input.
                Err(ParseError::Unexpected {
                    expected: Some(expected_str_from_char_range(self.start, self.end)),
                    found: None,
                })
            }
        }
    }

    /// Parses a variable number of elements.
    pub struct ParseCount<OutputType, ErrorType, ParserType: Parser<OutputType, ErrorType>> {
        /// The minimum count of elements to parse.
        min: usize,

        /// The maximum count of elements to parse (inclusively).
        max: usize,

        /// The type of parser to run for each element.
        parser: ParserType,

        /* Phantoms */
        _phantom1: PhantomData<OutputType>,
        _phantom2: PhantomData<ErrorType>,
    }

    impl<OutputType, ErrorType, ParserType: Parser<OutputType, ErrorType>>
        ParseCount<OutputType, ErrorType, ParserType>
    {
        /// Create a new count parser from the provided minimum and maximum counts.
        pub fn new(min: usize, max: usize, parser: ParserType) -> Self {
            Self {
                min,
                max,
                parser,
                _phantom1: PhantomData,
                _phantom2: PhantomData,
            }
        }
    }

    impl<OutputType, ErrorType, ParserType: Parser<OutputType, ErrorType>>
        Parser<Vec<OutputType>, ParseError> for ParseCount<OutputType, ErrorType, ParserType>
    {
        fn parse(&self, parser_state: ParserState) -> ParseResult<ParseError, Vec<OutputType>> {
            let mut new_state = parser_state;
            let mut output = Vec::with_capacity(self.min);

            // Keep parsing until enough elements are parsed.
            'parse_loop: loop {
                // If there are enough elements already, exit the loop.
                if output.len() >= self.max {
                    break 'parse_loop;
                }

                // Try to parse another element
                if let Ok((parsed_new_output, parsed_new_state)) =
                    self.parser.parse(new_state.clone())
                {
                    // If it succeeds, add the output to the output vec and
                    // update the state.
                    new_state = parsed_new_state;
                    output.push(parsed_new_output);
                } else {
                    // If it fails, break out of the loop.
                    break 'parse_loop;
                }
            }

            // Check if there are too few elements
            if output.len() < self.min {
                // If there aren't the right number of elements, construct and
                // return an error.
                Err(ParseError::WrongCount {
                    min: self.min,
                    max: self.max,
                    found: output.len(),
                })
            } else {
                // Otherwise, return the new state and the outputs.
                Ok((output, new_state))
            }
        }
    }

    /// Parses one element and then another element.
    pub struct ParseAnd<
        OutputTypeA,
        ErrorTypeA,
        ParserTypeA: Parser<OutputTypeA, ErrorTypeA>,
        OutputTypeB,
        ErrorTypeB,
        ParserTypeB: Parser<OutputTypeB, ErrorTypeB>,
    > {
        /// The first parser to run.
        parser_a: ParserTypeA,

        /// The second parser to run.
        parser_b: ParserTypeB,

        /* Phantom */
        _phantom: PhantomData<(OutputTypeA, ErrorTypeA, OutputTypeB, ErrorTypeB)>,
    }

    impl<
            OutputTypeA,
            ErrorTypeA,
            ParserTypeA: Parser<OutputTypeA, ErrorTypeA>,
            OutputTypeB,
            ErrorTypeB,
            ParserTypeB: Parser<OutputTypeB, ErrorTypeB>,
        > ParseAnd<OutputTypeA, ErrorTypeA, ParserTypeA, OutputTypeB, ErrorTypeB, ParserTypeB>
    {
        pub fn new(parser_a: ParserTypeA, parser_b: ParserTypeB) -> Self {
            Self {
                parser_a,
                parser_b,
                _phantom: PhantomData,
            }
        }
    }

    impl<
            ErrorType,
            OutputTypeA,
            ParserTypeA: Parser<OutputTypeA, ErrorType>,
            OutputTypeB,
            ParserTypeB: Parser<OutputTypeB, ErrorType>,
        > Parser<(OutputTypeA, OutputTypeB), ErrorType>
        for ParseAnd<OutputTypeA, ErrorType, ParserTypeA, OutputTypeB, ErrorType, ParserTypeB>
    {
        fn parse(
            &self,
            parser_state: ParserState,
        ) -> ParseResult<ErrorType, (OutputTypeA, OutputTypeB)> {
            // Run the first parser.
            let (a, new_state) = self.parser_a.parse(parser_state)?;

            // Run the second parser.
            let (b, new_state) = self.parser_b.parse(new_state)?;

            // Return the values.
            Ok(((a, b), new_state))
        }
    }
}

/// A trait to be added to other parsers that allows easier parser combining.
pub trait ParserExtensions<OutputType, ErrorType>: Parser<OutputType, ErrorType> {
    fn char(&self, character: char) -> ParseChar {
        ParseChar::from_char(character)
    }

    fn char_range(&self, start: char, end: char) -> ParseChar {
        ParseChar::from_range(start, end)
    }

    fn and<NextOutputType, NextErrorType, NextParserType: Parser<NextOutputType, NextErrorType>>(
        self,
        next: NextParserType,
    ) -> ParseAnd<OutputType, ErrorType, Self, NextOutputType, NextErrorType, NextParserType>
    where
        Self: Sized,
    {
        ParseAnd::new(self, next)
    }

    fn between(self, min: usize, max: usize) -> ParseCount<OutputType, ErrorType, Self>
    where
        Self: Sized,
    {
        ParseCount::new(min, max, self)
    }

    fn at_least(self, min: usize) -> ParseCount<OutputType, ErrorType, Self>
    where
        Self: Sized,
    {
        self.between(min, usize::max_value())
    }

    fn no_more_than(self, max: usize) -> ParseCount<OutputType, ErrorType, Self>
    where
        Self: Sized,
    {
        self.between(0, max)
    }

    fn optional(self) -> ParseCount<OutputType, ErrorType, Self>
    where
        Self: Sized,
    {
        self.no_more_than(1)
    }

    fn one_or_more(self) -> ParseCount<OutputType, ErrorType, Self>
    where
        Self: Sized,
    {
        self.at_least(1)
    }
}

impl<OutputType, ErrorType, ParserType: Parser<OutputType, ErrorType>>
    ParserExtensions<OutputType, ErrorType> for ParserType
{
}
