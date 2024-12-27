#[allow(clippy::enum_glob_use)]
use LexingStatus::*;

use super::end_state::end_current;
use super::escape::handle_escape;
use super::types::escape_state::EscapeStatus;
use super::types::lexing_data::LexingData;
use super::types::lexing_state::{CommentStatus, LexingStatus, SymbolStatus};
use super::types::tokens_types::Token;
use crate::errors::api::{Location, Res};

#[expect(clippy::too_many_lines)]
fn lex_char(
    ch: char,
    location: &Location,
    lex_data: &mut LexingData,
    lex_status: &mut LexingStatus,
    escape_status: &mut EscapeStatus,
    eol: bool,
) {
    match (ch, lex_status, escape_status) {
        (_, StartOfLine, _) if ch.is_whitespace() => (),
        /* Inside comment */
        ('/', status @ Comment(CommentStatus::Star), _) => {
            *status = Comment(CommentStatus::False);
        }
        ('*', status @ Comment(CommentStatus::True), _) => {
            *status = Comment(CommentStatus::Star);
        }
        (_, Comment(CommentStatus::True), _) => (),
        (_, status @ Comment(CommentStatus::Star), _) => {
            *status = Comment(CommentStatus::True);
        }
        /* Escaped character */
        (
            _,
            status @ (Char(None) | Str(_)),
            escape @ (EscapeStatus::Single | EscapeStatus::Sequence(_)),
        ) => {
            if let Some(escaped) = handle_escape(ch, lex_data, escape, location) {
                *escape = EscapeStatus::False;
                #[expect(clippy::wildcard_enum_match_arm)]
                match status {
                    Char(None) => *status = Char(Some(escaped)),
                    Str(val) => val.push(escaped),
                    _ => panic!("this can't happen, see match above"),
                }
            }
        }

        (_, _, EscapeStatus::Single | EscapeStatus::Sequence(_)) => {
            panic!("Can't happen because error raised on escape creation if wrong state.")
        }
        /* Create comment */
        ('*', status, _) if status.symbol().and_then(SymbolStatus::last) == Some('/') => {
            status.clear_last_symbol();
            end_current(status, lex_data, location);
            *status = Comment(CommentStatus::True);
        }

        /* Escape character */
        ('\\', Char(None) | Str(_), escape) => *escape = EscapeStatus::Single,
        ('\\', _, escape) if eol => *escape = EscapeStatus::Single,
        ('\\', state, _) => lex_data.push_err(location.to_error(format!(
            "Escape characters are only authorised in strings or chars, not in '{}' context.",
            state.repr(),
        ))),

        /* Static strings and chars */
        // open/close
        ('\'', status @ Char(_), _) => end_current(status, lex_data, location),
        ('\'', status, _) if !matches!(status, Str(_)) => {
            end_current(status, lex_data, location);
            *status = LexingStatus::Char(None);
        }
        ('\"', status @ Str(_), _) => {
            end_current(status, lex_data, location);
        }
        ('\"', status, _) if !matches!(status, Char(_)) => {
            end_current(status, lex_data, location);
            *status = LexingStatus::Str(String::new());
        }
        // middle
        (_, Char(Some(_)), _) => lex_data
            .push_err(location.to_error("A char must contain only one character.".to_owned())),
        (_, status @ Char(None), _) => *status = Char(Some(ch)),
        (_, Str(val), _) => val.push(ch),

        /* Operator symbols */
        ('/', status, _) if status.symbol().and_then(SymbolStatus::last) == Some('/') => {
            status.clear_last_symbol();
            end_current(status, lex_data, location);
            lex_data.set_end_line();
        }
        ('.', Identifier(ident), _) if !ident.contains('.') && ident.is_number() => {
            ident.push('.');
        }
        ('+' | '-', Identifier(ident), _)
            if !ident.contains('-') && !ident.contains('+') && ident.last_is_exp() =>
        {
            ident.push(ch);
        }
        (
            '(' | ')' | '[' | ']' | '{' | '}' | '~' | '!' | '*' | '&' | '%' | '/' | '>' | '<' | '='
            | '|' | '^' | ',' | '?' | ':' | ';' | '.' | '+' | '-',
            status,
            _,
        ) => {
            if let Symbols(symbol_status) = status {
                if let Some((size, symbol)) = symbol_status.push(ch) {
                    lex_data.push_token(Token::from_symbol(symbol, size, location));
                }
            } else {
                end_current(status, lex_data, location);
                *status = LexingStatus::Symbols(SymbolStatus::new(ch));
            }
        }

        /* Whitespace: end of everyone */
        (_, status, _) if ch.is_whitespace() => {
            end_current(status, lex_data, location);
        }

        // Whitespace: end of everyone
        (_, Identifier(val), _) if ch.is_alphanumeric() || matches!(ch, '_' | '.' | '+' | '-') => {
            // dbg!("here", &val, ch);
            val.push(ch);
            // dbg!("there", &val);
        }
        (_, status, _) if ch.is_alphanumeric() || matches!(ch, '_') => {
            if let Symbols(symb) = status
                && symb.last() == Some('.')
                && ch.is_ascii_digit()
            {
                symb.clear_last();
                end_current(status, lex_data, location);
                status.new_ident_str(format!("0.{ch}"));
            } else {
                end_current(status, lex_data, location);
                status.new_ident(ch);
            }
        }
        (_, status, _) => {
            lex_data.push_err(location.to_error(format!(
                "Character '{ch}' not supported in context of a '{}'.",
                status.repr(),
            )));
        }
    }
}

#[inline]
pub fn lex_file(content: &str, location: &mut Location) -> Res<Vec<Token>> {
    let mut lex_data = LexingData::default();
    let mut lex_status = LexingStatus::default();

    for line in content.lines() {
        lex_line(line, location, &mut lex_data, &mut lex_status);
        location.incr_line();
    }

    Res::from((lex_data.take_tokens(), lex_data.take_errors()))
}

fn lex_line(
    line: &str,
    location: &mut Location,
    lex_data: &mut LexingData,
    lex_status: &mut LexingStatus,
) {
    lex_data.newline();
    let mut escape_state = EscapeStatus::False;
    let trimed = line.trim_end();
    if trimed.is_empty() {
        return;
    }
    let last = trimed.len() - 1;
    for (idx, ch) in trimed.chars().enumerate() {
        lex_char(
            ch,
            location,
            lex_data,
            lex_status,
            &mut escape_state,
            idx == last,
        );
        location.incr_col();
        if lex_data.is_end_line() {
            break;
        }
    }
    end_current(lex_status, lex_data, location);
    if line.trim_end().ends_with('\\') {
        if line.ends_with(char::is_whitespace) {
            lex_data.push_err(location.to_suggestion(
                "found white space after '\\' at EOL. Please remove the space.".to_owned(),
            ));
        }
    } else {
        *lex_status = LexingStatus::default();
    }
}