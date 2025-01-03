//! Module to parse octal-represented number constants

use super::super::macros::parse_int_from_radix;
use super::super::parse::OverParseRes;
use super::super::types::arch_types::{Int, Long, LongLong, UInt, ULong, ULongLong};
use super::super::types::{ERR_PREFIX, Number, NumberType};
use crate::errors::api::Location;

/// Parses an octal value.
///
/// The input doesn't contain the prefix ('0') or the suffix (e.g. 'ULL').
///
/// # Returns
///
/// A [`OverParseRes`]. It contains one or more of the following:
///
///  - the value, if the parsing succeeded
///  - errors, if there are some
///  - overflow warning if a value was crapped to fit in the specified type.
///
///
/// # Examples
///
/// ```ignore
/// use crate::errors::location::Location;
/// use crate::lexer::numbers::parse::OverParseRes;
/// use crate::lexer::numbers::types::{Number, NumberType};
///
/// assert!(
///     to_oct_value("123", &NumberType::Int, &Location::from(String::new()))
///         == OverParseRes::Value(Number::Int(83))
/// );
/// assert!(
///     to_oct_value(
///         "377",
///         &NumberType::Int,
///         &Location::from(String::new())
///     ) == OverParseRes::ValueOverflow(2i32.pow(31) - 1)
/// );
/// assert!(matches!(
///     to_oct_value("1f3", &NumberType::Int, &Location::from(String::new())),
///     OverParseRes::Err(_)
/// ));
/// ```
pub fn to_oct_value(
    literal: &str,
    nb_type: &NumberType,
    location: &Location,
) -> OverParseRes<Number> {
    if literal.chars().all(|ch| matches!(ch, '0'..='7')) {
        parse_int_from_radix!(
            location,
           nb_type, literal, "an octal must be an integer", 8, Int Long LongLong UInt ULong ULongLong
        )
    } else {
        let first = literal
            .chars()
            .find(|ch| matches!(ch, '0'..='7'))
            .expect("Exists according to line above");
        OverParseRes::from(location.to_failure(format!("{ERR_PREFIX}a octal constant must only contain digits between '0' and '7'. Found invalid character '{first}'.")))
    }
}
