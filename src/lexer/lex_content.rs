#[allow(clippy::enum_glob_use)]
use LexingState::*;

use super::state::api::{
    end_current, handle_escape, CommentState, EscapeState, LexingState, SymbolState
};
use super::types::api::{LexingData, Token};
use crate::errors::api::{Location, Res};

#[expect(clippy::too_many_lines)]
fn lex_char(
    ch: char,
    location: &Location,
    lex_data: &mut LexingData,
    lex_state: &mut LexingState,
    escape_state: &mut EscapeState,
    eol: bool,
) {
    match (ch, lex_state, escape_state) {
        (_, StartOfLine, _) if ch.is_whitespace() => (),
        /* Inside comment */
        ('/', state @ Comment(CommentState::Star), _) => {
            *state = Comment(CommentState::False);
        }
        ('*', state @ Comment(CommentState::True), _) => {
            *state = Comment(CommentState::Star);
        }
        (_, Comment(CommentState::True), _) => (),
        (_, state @ Comment(CommentState::Star), _) => {
            *state = Comment(CommentState::True);
        }
        /* Escaped character */
        (
            _,
            state @ (Char(None) | Str(_)),
            escape @ (EscapeState::Single | EscapeState::Sequence(_)),
        ) => {
            if let Some(escaped) = handle_escape(ch, lex_data, escape, location) {
                *escape = EscapeState::False;
                #[expect(clippy::wildcard_enum_match_arm)]
                match state {
                    Char(None) => *state = Char(Some(escaped)),
                    Str(val) => val.push(escaped),
                    _ => panic!("this can't happen, see match above"),
                }
            }
        }

        (_, _, EscapeState::Single | EscapeState::Sequence(_)) => {
            panic!("Can't happen because error raised on escape creation if wrong state.")
        }
        /* Create comment */
        ('*', state, _) if state.symbol().and_then(SymbolState::last) == Some('/') => {
            state.clear_last_symbol();
            end_current(state, lex_data, location);
            *state = Comment(CommentState::True);
        }

        /* Escape character */
        ('\\', Char(None) | Str(_), escape) => *escape = EscapeState::Single,
        ('\\', _, escape) if eol => *escape = EscapeState::Single,
        ('\\', state, _) => lex_data.push_err(location.to_error(format!(
            "Escape characters are only authorised in strings or chars, not in '{}' context.",
            state.repr(),
        ))),

        /* Static strings and chars */
        // open/close
        ('\'', state @ Char(_), _) => end_current(state, lex_data, location),
        ('\'', state, _) if !matches!(state, Str(_)) => {
            end_current(state, lex_data, location);
            *state = LexingState::Char(None);
        }
        ('\"', state @ Str(_), _) => {
            end_current(state, lex_data, location);
        }
        ('\"', state, _) if !matches!(state, Char(_)) => {
            end_current(state, lex_data, location);
            *state = LexingState::Str(String::new());
        }
        // middle
        (_, Char(Some(_)), _) => lex_data
            .push_err(location.to_error("A char must contain only one character.".to_owned())),
        (_, state @ Char(None), _) => *state = Char(Some(ch)),
        (_, Str(val), _) => val.push(ch),

        /* Operator symbols */
        ('/', state, _) if state.symbol().and_then(SymbolState::last) == Some('/') => {
            state.clear_last_symbol();
            end_current(state, lex_data, location);
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
            state,
            _,
        ) => {
            if let Symbols(symbol_state) = state {
                let (value, error) = symbol_state.push(ch);
                if let Some(msg) = error {
                    lex_data.push_err(location.to_warning(msg));
                }
                if let Some((size, symbol)) = value {
                    lex_data.push_token(Token::from_symbol(symbol, size, location));
                }
            } else {
                end_current(state, lex_data, location);
                *state = LexingState::Symbols(SymbolState::new(ch));
            }
        }

        /* Whitespace: end of everyone */
        (_, state, _) if ch.is_whitespace() => {
            end_current(state, lex_data, location);
        }

        // Whitespace: end of everyone
        (_, Identifier(val), _) if ch.is_alphanumeric() || matches!(ch, '_' | '.' | '+' | '-') => {
            // dbg!("here", &val, ch);
            val.push(ch);
            // dbg!("there", &val);
        }
        (_, state, _) if ch.is_alphanumeric() || matches!(ch, '_') => {
            if let Symbols(symb) = state
                && symb.last() == Some('.')
                && ch.is_ascii_digit()
            {
                symb.clear_last();
                end_current(state, lex_data, location);
                state.new_ident_str(format!("0.{ch}"));
            } else {
                end_current(state, lex_data, location);
                state.new_ident(ch);
            }
        }
        (_, state, _) => {
            lex_data.push_err(location.to_error(format!(
                "Character '{ch}' not supported in context of a '{}'.",
                state.repr(),
            )));
        }
    }
}

#[inline]
pub fn lex_file(content: &str, location: &mut Location) -> Res<Vec<Token>> {
    let mut lex_data = LexingData::default();
    let mut lex_state = LexingState::default();

    for line in content.lines() {
        lex_line(line, location, &mut lex_data, &mut lex_state);
        location.incr_line();
    }

    Res::from((lex_data.take_tokens(), lex_data.take_errors()))
}

fn lex_line(
    line: &str,
    location: &mut Location,
    lex_data: &mut LexingData,
    lex_state: &mut LexingState,
) {
    lex_data.newline();
    let mut escape_state = EscapeState::False;
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
            lex_state,
            &mut escape_state,
            idx == last,
        );
        location.incr_col();
        if lex_data.is_end_line() {
            break;
        }
    }
    if escape_state != EscapeState::Single {
        end_current(lex_state, lex_data, location);
    }
    if line.trim_end().ends_with('\\') {
        if line.ends_with(char::is_whitespace) {
            lex_data.push_err(location.to_suggestion(
                "found white space after '\\' at EOL. Please remove the space.".to_owned(),
            ));
        }
    } else {
        *lex_state = LexingState::default();
    }
}
