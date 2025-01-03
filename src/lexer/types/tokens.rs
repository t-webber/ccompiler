//! Module to define a token
//!
//! This module contains the definition of [`Token`] and [`TokenValue`], used to
//! store and pass on the values of the token that were lexed. They are stored
//! in [`LexingData`] during lexing and then returned.

use core::str::pattern;
use core::{fmt, mem};

use super::super::numbers::api::Number;
use super::super::types::api::LexingData;
use super::keywords::{Keyword, TryKeyword};
use super::symbols::Symbol;
use crate::errors::api::Location;

/// Represents an identifier
///
/// An identifier is a token that contains a succession of alphanumeric digits
/// (or underscores).
///
/// Identifiers are used as variable names, custom types, number constants etc.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ident(String);

impl Ident {
    /// Checks if the underlying string contains a pattern
    pub fn contains<P: pattern::Pattern>(&self, pat: P) -> bool {
        self.0.contains(pat)
    }

    /// Returns the first character of the underlying string
    pub fn first(&self) -> Option<char> {
        self.0.chars().next()
    }

    /// Checks if the underlying string is empty
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Checks if the first character is a valid ascii digit (`[0-9]`).
    pub fn is_number(&self) -> bool {
        self.first().unwrap_or('x').is_ascii_digit()
    }

    /// Checks if last character of the string
    pub fn last_is_exp(&self) -> bool {
        self.is_number()
            && match self.0.chars().last() {
                Some('p' | 'P') => self.0.starts_with("0x"),
                Some('e' | 'E') => !self.0.starts_with("0x"), /* if the number expression starts with 0 and contains an exponent, the number is considered decimal, not octal. */
                Some(_) | None => false,
            }
    }

    /// Returns the length of the underlying string
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Pushes a character to the underlying string
    pub fn push(&mut self, ch: char) {
        self.0.push(ch);
    }

    /// Takes the value of the underlying string
    pub fn take_value(&mut self) -> String {
        mem::take(&mut self.0)
    }

    /// Returns a reference to the underlying string
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl From<String> for Ident {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Struct that stores a lexed token
#[derive(Debug)]
pub struct Token {
    /// Location of the token
    ///
    /// The location is stored with the token to have it when parsing.
    location: Location,
    /// Value of the token
    value: TokenValue,
}

impl Token {
    /// Converts a `char` into a token whose value is a [`TokenValue::Char`]
    pub(crate) fn from_char(ch: char, location: &Location) -> Self {
        Self {
            value: TokenValue::Char(ch),
            location: location.to_owned().into_past_with_length(1),
        }
    }

    /// Converts an identifier into a token whose value is a
    /// [`TokenValue::Ident`] or [`TokenValue::Keyword`] depending on the value
    /// of the identifier.
    pub(crate) fn from_identifier(
        lex_data: &mut LexingData,
        literal: &mut Ident,
        location: &Location,
    ) -> Self {
        let len = literal.len();
        let value = literal.take_value();
        let token_value = match Keyword::from_value_or_res(&value) {
            TryKeyword::Success(keyword) => TokenValue::Keyword(keyword),
            TryKeyword::Deprecated(keyword) => {
                let new_keyword = value
                    .char_indices()
                    .filter_map(|(idx, ch)| {
                        if idx == 0 {
                            None
                        } else if idx == 1 {
                            Some(ch.to_ascii_lowercase())
                        } else {
                            Some(ch)
                        }
                    })
                    .collect::<String>();
                lex_data.push_err(location.to_owned().into_past_with_length(len).to_warning(format!("Underscore operators are deprecated since C23. Consider using the new keyword: {new_keyword}")));
                TokenValue::Keyword(keyword)
            }
            TryKeyword::Failure => TokenValue::Ident(value),
        };
        Self {
            location: location.to_owned().into_past_with_length(len),
            value: token_value,
        }
    }

    /// Converts a [`Number`] into a token whose value is a
    /// [`TokenValue::Number`].
    pub(crate) fn from_number(number: Number, location: &Location) -> Self {
        Self {
            value: TokenValue::Number(number),
            location: location.to_owned(),
        }
    }

    /// Converts a string constant into a token whose value is a
    /// [`TokenValue::Str`]
    pub(crate) fn from_str(str: String, location: &Location) -> Self {
        Self {
            location: location.to_owned().into_past_with_length(str.len()),
            value: TokenValue::Str(str),
        }
    }

    /// Converts a [`Symbol`] into a token whose value is a
    /// [`TokenValue::Symbol`].
    pub(crate) fn from_symbol(symbol: Symbol, size: usize, location: &Location) -> Self {
        Self {
            value: TokenValue::Symbol(symbol),
            location: location.to_owned().into_past_with_length(size),
        }
    }

    /// Returns a reference to the value of the [`Token`]
    #[inline]
    #[must_use]
    pub const fn get_value(&self) -> &TokenValue {
        &self.value
    }

    /// Returns a mutable reference to the value of the [`Token`]
    pub(crate) const fn get_value_mut(&mut self) -> &mut TokenValue {
        &mut self.value
    }

    /// Returns the value and the location of the [`Token`]
    pub(crate) fn into_value_location(self) -> (TokenValue, Location) {
        (self.value, self.location)
    }
}

#[expect(clippy::min_ident_chars)]
impl fmt::Display for Token {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

/// Enum that contains the value of the Token.
#[derive(PartialEq, Debug)]
pub enum TokenValue {
    /// Chars
    ///
    /// # Rules
    ///
    /// - Delimited with single quotes `'`
    /// - Contain a single character.
    ///
    /// # Examples
    ///
    /// `'o'` and `'\u2205'`
    Char(char),
    /// Identifiers
    ///
    /// # Rules
    ///
    /// - Contain only alphanumeric characters and underscores
    /// - Don't start with a numeral digit.
    ///
    /// # Examples
    ///
    /// `_Hello` and `STRUCT_NAME`.
    Ident(String),
    /// Keywords
    ///
    /// # Rules
    ///
    /// See [CppReference](https://en.cppreference.com/w/c/keyword) for the list of C keywords.
    ///
    /// # Examples
    ///
    /// `const`, `int`, `sizeof`, `thread_local`
    Keyword(Keyword),
    /// Number constants
    ///
    /// # Rules
    ///
    /// See [`Number`] for the list of rules
    ///
    /// # Examples
    ///
    /// `0xfe.d2p-9`, `0123`
    Number(Number),
    /// String constants
    ///
    /// # Rules
    ///
    /// - Delimited by double quotes
    /// - Successive quotes are merged
    ///
    /// # Examples
    ///
    /// `""`, `"Hello world"` and `"Hello""World"`
    Str(String),
    /// Symbols
    ///
    /// # Rules
    ///
    /// - All characters that don't fit in the other types.
    ///
    /// # Examples
    ///
    /// `<<=`, `+`, `[`
    Symbol(Symbol),
}

#[expect(clippy::min_ident_chars, clippy::use_debug)]
impl fmt::Display for TokenValue {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char(arg0) => write!(f, "'{arg0}'"),
            Self::Keyword(arg0) => write!(f, "Keyword({arg0})"),
            Self::Number(arg0) => write!(f, "{arg0}"),
            Self::Symbol(arg0) => write!(f, "{arg0:?}"),
            Self::Ident(arg0) => write!(f, "Ident({arg0})"),
            Self::Str(arg0) => write!(f, "\"{arg0}\""),
        }
    }
}
