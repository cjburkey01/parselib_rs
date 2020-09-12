mod chars {
    use crate::parsers::utf8::ParseChar;
    use crate::parsers::ParserExtensions;
    use crate::{ParseError, Parser, ParserState};

    fn test_char_parser(
        char_parser: ParseChar,
        test_string: &str,
        expected_c: char,
        expected_input: &str,
        expected_index: usize,
    ) {
        let parser_start_state = ParserState::new(test_string);

        // Try to parse the input.
        let val = char_parser.parse(parser_start_state);

        // Check if it matches the expected result.
        if let Ok((c, ParserState { input, index })) = val {
            assert_eq!(c, expected_c);
            assert_eq!(input, expected_input);
            assert_eq!(index, expected_index);
        } else {
            panic!("parsing error: {}", val.unwrap_err());
        }
    }

    #[test]
    fn char_parser_a() {
        // Create the parser to match the character 'h'.
        let char_parser = ParseChar::from_char('h');

        // Create the initial parser state.
        let test_string = "hello world!";

        // Test the parser.
        test_char_parser(char_parser, test_string, 'h', "ello world!", 1);
    }

    #[test]
    fn char_parser_b() {
        // Create the parser to match any character from 'a' to 'z'.
        let char_parser = crate::parsers::utf8::ParseChar::from_range('a', 'z');

        // Create the initial parser state.
        let test_string = "hello world!";

        // Test the parser.
        test_char_parser(char_parser, test_string, 'h', "ello world!", 1);
    }

    #[test]
    fn char_parser_c() {
        // Create the parser to match any character from 'i' to 'z'.
        let char_parser = ParseChar::from_range('i', 'z');

        // Create the initial parser state.
        let test_string = "hello world!";

        // Create the initial parser state.
        let parser_start_state = ParserState::new(test_string);

        // Try to parse the input.
        let val = char_parser.parse(parser_start_state);

        // Check the values.
        match val {
            // Should be an error.
            Err(ParseError::Unexpected {
                expected: Some(expected),
                found: Some(found),
            }) => {
                // The error should contain this info.
                if &expected != "i..z" || &found != "h" {
                    panic!(
                        "parse failed but has incorrect error | found: \"{}\" | expected: {}",
                        found, expected
                    )
                }
            }

            // Wrong kind of error?
            Err(e) => panic!("parse error: {}", e),

            // If the parser succeeds, this test fails.
            Ok((char_at, new_state)) => panic!(
                "parse succeeded but meant to fail: found char: '{}' | final state: {:?}",
                char_at, new_state
            ),
        }
    }
}

mod counts {
    use crate::parsers::utf8::ParseChar;
    use crate::parsers::ParserExtensions;
    use crate::{Parser, ParserState};

    #[test]
    fn count_parser_a() {
        // Create the parser to match any character from 'a' to 'z' and a
        // parser to accept 0-100 characters between (inclusively) 'a' and 'z'.
        let count_parser = ParseChar::from_range('a', 'z').no_more_than(100);

        // Create the initial parser state.
        let test_string = "hello world";
        let parser_start_state = ParserState::new(test_string);

        // Perform the parsing
        let result = count_parser.parse(parser_start_state);

        // Check the output
        match result {
            Ok((chars, new_state)) => {
                assert_eq!(chars, "hello".chars().collect::<Vec<char>>());
                assert_eq!(new_state.input, String::from(" world"));
                assert_eq!(new_state.index, "hello".len());
            }
            Err(e) => panic!("unexpected parsing error: {}", e),
        }
    }
}
