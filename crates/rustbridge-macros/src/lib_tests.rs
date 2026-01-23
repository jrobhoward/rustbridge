#![allow(non_snake_case)]

use super::*;

// to_snake_case tests

#[test]
fn to_snake_case___pascal_case___converts_correctly() {
    assert_eq!(to_snake_case("CreateUserRequest"), "create_user_request");
}

#[test]
fn to_snake_case___acronym___splits_each_letter() {
    assert_eq!(to_snake_case("UserID"), "user_i_d");
}

#[test]
fn to_snake_case___single_word___lowercases() {
    assert_eq!(to_snake_case("Simple"), "simple");
}

#[test]
fn to_snake_case___all_uppercase___splits_all() {
    assert_eq!(to_snake_case("ABC"), "a_b_c");
}

#[test]
fn to_snake_case___already_lowercase___unchanged() {
    assert_eq!(to_snake_case("simple"), "simple");
}

#[test]
fn to_snake_case___empty_string___returns_empty() {
    assert_eq!(to_snake_case(""), "");
}
