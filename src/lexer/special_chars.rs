use super::lexing_state::{EscapeSequence, EscapeStatus, ParsingState};
use super::numbers::literal_to_number;
use super::types::Token;
use crate::errors::location::Location;
use crate::lexer::types::TokenValue;
use crate::{safe_parse_int, to_error};
use core::mem;

pub fn end_both(lex_state: &mut ParsingState, location: &Location) {
    end_operator(lex_state, location);
    end_literal(lex_state, location);
}

fn end_unicode_sequence(
    lex_state: &mut ParsingState,
    value: &str,
    location: &Location,
) -> Result<(), ()> {
    match crate::safe_parse_int!(
        "Invalid escaped unicode number: ",
        u32,
        location,
        u32::from_str_radix(value, 16)
    )
    .map(char::from_u32)
    {
        Err(err) => lex_state.push_err(err),
        Ok(Some(ch)) => {
            lex_state.literal.push(ch);
            return Ok(());
        }
        Ok(None) => lex_state.push_err(to_error!(
            location,
            "Invalid escaped unicode number: {} is not a valid unicode character.",
            value
        )),
    }
    Err(())
}

fn expect_min_length(
    lex_state: &mut ParsingState,
    size: usize,
    value: &str,
    location: &Location,
    sequence: &EscapeSequence,
) -> Result<(), ()> {
    let len = value.len();
    if len < size {
        lex_state.push_err(to_error!(
            location,
            "Invalid escaped {} number: must contain 4 digits, but found only {}",
            sequence.repr(),
            len
        ));
        return Err(());
    }
    Ok(())
}

fn expect_max_length(size: usize, value: &str) {
    assert!(value.len() <= size, "Never should have pushed here");
}

pub fn end_escape_sequence(lex_state: &mut ParsingState, location: &Location) -> Result<(), ()> {
    let sequence = lex_state.escape.get_unsafe_sequence();
    match &sequence {
        EscapeSequence::ShortUnicode(ref value) => {
            expect_max_length(4, value);
            expect_min_length(lex_state, 4, value, location, &sequence)?;
            end_unicode_sequence(lex_state, value, location)?;
        }
        EscapeSequence::Unicode(ref value) => {
            if value.len() <= 4 {
                lex_state.push_err(to_error!(
                    location,
                    "Invalid escaped unicode number: An escaped big unicode must contain 8 hexadecimal digits, found only {}. Did you mean to use lowercase \\u?",
                    value.len()
                ));
                Err(())?;
            }
            expect_max_length(8, value);
            expect_min_length(lex_state, 8, value, location, &sequence)?;
            end_unicode_sequence(lex_state, value, location)?;
        }
        EscapeSequence::Hexadecimal(ref value) => {
            expect_max_length(3, value);
            expect_min_length(lex_state, 2, value, location, &sequence)?;
            let int =
                u8::from_str_radix(value, 16).expect("We push only numeric so this doesn't happen");
            lex_state.literal.push(int.into());
        }
        EscapeSequence::Octal(ref value) => {
            expect_max_length(3, value);
            expect_min_length(lex_state, 1, value, location, &sequence)?;
            match safe_parse_int!(
                "Invalid octal escape sequence :",
                u32,
                location,
                u32::from_str_radix(value, 8)
            ) {
                Ok(int) if value.len() < 3 || int <= 0o377 => {
                    #[allow(
                        clippy::as_conversions,
                        clippy::cast_possible_truncation,
                        reason = "int <= 255"
                    )]
                    lex_state.literal.push(char::from(int as u8));
                }
                Ok(_) => {
                    #[allow(clippy::string_slice, reason = "len = 3")]
                    match safe_parse_int!(
                        "Invalid octal escape sequence: ",
                        u8,
                        location,
                        u8::from_str_radix(&value[0..2], 8)
                    ) {
                        Ok(int2) => {
                            lex_state.literal.push(char::from(int2));
                            #[allow(clippy::indexing_slicing, reason = "len = 3")]
                            lex_state.literal.push(char::from(value.as_bytes()[2]));
                        }
                        Err(err) => lex_state.push_err(err),
                    }
                }
                Err(err) => lex_state.push_err(err),
            }
        }
    };
    lex_state.escape = EscapeStatus::Trivial(false);
    Ok(())
}

#[allow(clippy::needless_pass_by_ref_mut, clippy::todo)]
fn end_literal(lex_state: &mut ParsingState, location: &Location) {
    if !lex_state.literal.is_empty() {
        let possible_number = literal_to_number(lex_state, location);
        match possible_number {
            None => {
                let token = Token::from_identifier(mem::take(&mut lex_state.literal), location);
                lex_state.push_token(token);
            }
            Some(nb) => {
                let token = Token::from_number(nb, location);
                lex_state.push_token(token);
            }
        }
    }
}

pub fn end_operator(lex_state: &mut ParsingState, location: &Location) {
    let mut idx: usize = 0;
    while !lex_state.is_empty() && idx <= 2 {
        idx += 1;
        if let Some((size, symbol)) = lex_state.try_to_operator() {
            let token = Token::from_symbol(symbol, size, location);
            lex_state.push_token(token);
        } else {
            panic!(
                "This can't happen, as lex_state is not empty! ParsingState: {:?}",
                &lex_state
            );
        }
    }
    assert!(lex_state.is_empty(), "Not possible: executing 3 times the conversion, with stritcly decreasing number of non empty elements! This can't happen. ParsingState: {:?}", &lex_state);
}

fn end_string(lex_state: &mut ParsingState, location: &Location) {
    if !lex_state.literal.is_empty() {
        if let Some(last_token) = lex_state.pop_token() {
            if let TokenValue::Str(last_str) = last_token.into_value() {
                let new_token =
                    Token::from_str(last_str + &mem::take(&mut lex_state.literal), location);
                lex_state.push_token(new_token);
                return;
            }
        }
        let token = Token::from_str(mem::take(&mut lex_state.literal), location);
        lex_state.push_token(token);
    }
    assert!(lex_state.literal.is_empty(), "Not possible: The string was just cleared, except if i am stupid and take doesn't clear ??!! ParsingState:{:?}", &lex_state);
}

pub fn handle_double_quotes(lex_state: &mut ParsingState, location: &Location) {
    if lex_state.double_quote {
        end_string(lex_state, location);
        lex_state.double_quote = false;
    } else {
        end_both(lex_state, location);
        lex_state.double_quote = true;
    }
}

pub fn handle_escaped(ch: char, lex_state: &mut ParsingState, location: &Location) {
    match &lex_state.escape {
        EscapeStatus::Sequence(_) => handle_escaped_sequence(ch, lex_state, location),
        EscapeStatus::Trivial(_) => handle_one_escaped_char(ch, lex_state, location),
    }
}

fn handle_escaped_sequence(ch: char, lex_state: &mut ParsingState, location: &Location) {
    let escape_sequence = lex_state.escape.get_unsafe_sequence();
    if !ch.is_ascii_hexdigit() || (escape_sequence.is_octal() && !ch.is_ascii_octdigit()) {
        if end_escape_sequence(lex_state, location).is_ok() {
            lex_state.literal.push(ch);
        }
    } else {
        let value = lex_state.escape.get_unsafe_sequence_value_mut();
        value.push(ch);
        if value.len() == escape_sequence.max_len() {
            let _e = end_escape_sequence(lex_state, location);
        }
    }
}

fn handle_one_escaped_char(ch: char, lex_state: &mut ParsingState, location: &Location) {
    lex_state.escape = EscapeStatus::Trivial(false);
    if lex_state.double_quote || lex_state.single_quote {
        match ch {
            '\0' => lex_state.literal.push('\0'),
            'a' => lex_state.literal.push('\u{0007}'), // alert (bepp, bell)
            'b' => lex_state.literal.push('\u{0008}'), // backspace
            't' => lex_state.literal.push('\u{0009}'), // horizontal tab
            'n' => lex_state.literal.push('\u{000A}'), // newline (line feed)
            'v' => lex_state.literal.push('\u{000B}'), // vertical tab
            'f' => lex_state.literal.push('\u{000C}'), // formfeed page break
            'r' => lex_state.literal.push('\u{000D}'), // carriage return
            'e' => lex_state.literal.push('\u{001B}'), // escape character
            '"' => lex_state.literal.push('\u{0022}'), // double quotation mark
            '\'' => lex_state.literal.push('\u{0027}'), // apostrophe or single quotiation mark
            '?' => lex_state.literal.push('\u{003F}'), // question mark (used to avoid tigraphs)
            '\\' => lex_state.literal.push('\u{005C}'), // backslash
            'u' => {
                lex_state.escape =
                    EscapeStatus::Sequence(EscapeSequence::ShortUnicode(String::new()));
            }
            'U' => {
                lex_state.escape = EscapeStatus::Sequence(EscapeSequence::Unicode(String::new()));
            }
            'x' | 'h' => {
                lex_state.escape =
                    EscapeStatus::Sequence(EscapeSequence::Hexadecimal(String::new()));
            }
            _ if ch.is_numeric() => {
                lex_state.escape = EscapeStatus::Sequence(EscapeSequence::Octal(ch.to_string()));
            }
            _ => lex_state.push_err(to_error!(
                location,
                "Character '{ch}' can not be escaped, even inside a string or a char."
            )),
        }
    } else {
        lex_state.push_err(to_error!(
            location,
            "\\ escape character can only be used inside a string or char to espace a character."
        ));
    }
}

pub fn handle_single_quotes(lex_state: &mut ParsingState, location: &Location) {
    if lex_state.single_quote {
        assert!(lex_state.literal.len() == 1, "Never should have pushed");
        let ch = lex_state.literal.chars().next().expect("len = 1");
        lex_state.push_token(Token::from_char(ch, location));
        lex_state.single_quote = false;
    } else {
        end_both(lex_state, location);
        lex_state.single_quote = true;
    }
}

pub fn handle_symbol(ch: char, lex_state: &mut ParsingState, location: &Location) {
    end_literal(lex_state, location);
    if let Some((size, symbol)) = lex_state.push(ch) {
        let token = Token::from_symbol(symbol, size, location);
        lex_state.push_token(token);
    }
}
