#![allow(non_snake_case)]

use super::*;

// to_pascal_case tests

#[test]
fn to_pascal_case___kebab_case___converts() {
    assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
}

#[test]
fn to_pascal_case___snake_case___converts() {
    assert_eq!(to_pascal_case("my_plugin"), "MyPlugin");
}

#[test]
fn to_pascal_case___single_word___capitalizes() {
    assert_eq!(to_pascal_case("simple"), "Simple");
}

#[test]
fn to_pascal_case___already_pascal___unchanged() {
    assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
}

#[test]
fn to_pascal_case___empty___returns_empty() {
    assert_eq!(to_pascal_case(""), "");
}

// tag_to_method_name tests

#[test]
fn tag_to_method_name___dotted___converts_to_underscores() {
    assert_eq!(tag_to_method_name("user.create"), "user_create");
}

#[test]
fn tag_to_method_name___deeply_nested___converts_all_dots() {
    assert_eq!(tag_to_method_name("order.item.add"), "order_item_add");
}

#[test]
fn tag_to_method_name___with_dashes___converts_dashes() {
    assert_eq!(tag_to_method_name("my-tag"), "my_tag");
}

#[test]
fn tag_to_method_name___simple___unchanged() {
    assert_eq!(tag_to_method_name("echo"), "echo");
}
