pub mod api {
    #![allow(clippy::pub_use)]

    pub use super::parse_content::parse_tokens;
}

mod keyword;
mod modifiers;
mod parse_content;
mod state;
mod symbols;
mod types;

use core::fmt;

use crate::EMPTY;

#[expect(clippy::ref_option)]
fn repr_option<T: fmt::Display>(opt: &Option<T>) -> String {
    opt.as_ref().map_or_else(|| EMPTY.to_owned(), T::to_string)
}

fn repr_vec<T: fmt::Display>(vec: &[T]) -> String {
    vec.iter()
        .map(|node| format!("{node}"))
        .collect::<Vec<_>>()
        .join(", ")
}
