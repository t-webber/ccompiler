use core::num::ParseFloatError;
use core::str::FromStr;

use crate::errors::compile::CompileError;
use crate::errors::location::Location;
#[allow(clippy::wildcard_imports)]
use crate::lexer::numbers::types::arch_types::*;
use crate::lexer::numbers::types::{Number, NumberType, ERR_PREFIX};
use crate::lexer::numbers::OverParseRes;

macro_rules! parse_number {
    ($location:ident, $nb_type:ident, $literal:tt, $($int:ident)*, $($float:ident)*) => {
        match $nb_type {
            NumberType::LongDouble => OverParseRes::from($location.to_error(format!("{ERR_PREFIX}`long double` not supported yet."))), //TODO: f128 not implemented
            $(NumberType::$int => $crate::lexer::numbers::parse::safe_parse_int!(ERR_PREFIX, $int, $location, $literal.parse::<$int>()).map(|nb| Number::$int(nb)),)*
            $(NumberType::$float => OverParseRes::from(parse_and_error::<$float>($literal, $location).map(|nb| Number::$float(nb))?),)*
        }
    };
}

fn parse_and_error<T>(literal: &str, location: &Location) -> Result<T, CompileError>
where
    T: FromStr,
    <T as FromStr>::Err: Into<ParseFloatError>,
{
    literal
        .parse::<T>()
        .map_err(|_err| location.to_error(format!("{ERR_PREFIX}invalid decimal float number.")))
}

pub fn to_decimal_value(
    literal: &str,
    nb_type: &NumberType,
    location: &Location,
) -> OverParseRes<Number> {
    parse_number!(location,  nb_type, literal, Int Long LongLong UInt ULong ULongLong, Float Double )
}
