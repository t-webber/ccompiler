use expressions::prelude::*;
use std::fs;

const PREFIX: &str = "./tests/data/lexer-";

#[expect(clippy::unwrap_used)]
fn test_lexer_on_file(file: &str) {
    let path = format!("{PREFIX}{file}.c");
    let content = fs::read_to_string(&path).unwrap();
    let mut location = Location::from(path.clone());
    let Res { errors, .. } = lex_file(&content, &mut location);
    if !errors.is_empty() {
        display_errors(errors, &[(path, &content)], "lexer");
        panic!()
    }
}

#[test]
fn lexer_escape() {
    test_lexer_on_file("escape");
}

#[test]
fn lexer_general() {
    test_lexer_on_file("general");
}

#[test]
fn lexer_numbers_1() {
    test_lexer_on_file("numbers-1");
}

#[test]
fn lexer_numbers_2() {
    test_lexer_on_file("numbers-2");
}
