//! Naming convention utilities for code generation.
//!
//! This module provides functions for converting between different naming conventions
//! used across Rust, Java, Kotlin, and other target languages.
//!
//! # Supported Conversions
//!
//! | Input | Function | Output |
//! |-------|----------|--------|
//! | `snake_case` | [`to_camel_case`] | `camelCase` |
//! | `snake_case` | [`to_pascal_case`] | `PascalCase` |
//! | `word` | [`capitalize`] | `Word` |
//! | `dot.separated` | [`to_method_name`] | `dot_separated` |

/// Convert snake_case to camelCase.
///
/// # Examples
///
/// ```
/// use rustbridge_cli::codegen::naming::to_camel_case;
///
/// assert_eq!(to_camel_case("hello_world"), "helloWorld");
/// assert_eq!(to_camel_case("display_name"), "displayName");
/// assert_eq!(to_camel_case("already"), "already");
/// ```
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push_str(&c.to_uppercase().to_string());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a string to PascalCase.
///
/// Handles snake_case, kebab-case, and already-capitalized input.
///
/// # Examples
///
/// ```
/// use rustbridge_cli::codegen::naming::to_pascal_case;
///
/// assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
/// assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
/// assert_eq!(to_pascal_case("hello"), "Hello");
/// ```
#[allow(dead_code)] // Used by tests and future generators
pub fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Capitalize the first letter of a string.
///
/// # Examples
///
/// ```
/// use rustbridge_cli::codegen::naming::capitalize;
///
/// assert_eq!(capitalize("hello"), "Hello");
/// assert_eq!(capitalize("world"), "World");
/// assert_eq!(capitalize(""), "");
/// ```
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Convert a type tag or identifier to a method name.
///
/// Replaces dots and dashes with underscores.
///
/// # Examples
///
/// ```
/// use rustbridge_cli::codegen::naming::to_method_name;
///
/// assert_eq!(to_method_name("user.create"), "user_create");
/// assert_eq!(to_method_name("get-user"), "get_user");
/// ```
#[allow(dead_code)] // Used by tests and future generators
pub fn to_method_name(tag: &str) -> String {
    tag.replace(['.', '-'], "_")
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn to_camel_case___converts_snake_case() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("display_name"), "displayName");
        assert_eq!(to_camel_case("foo_bar_baz"), "fooBarBaz");
    }

    #[test]
    fn to_camel_case___handles_simple_words() {
        assert_eq!(to_camel_case("simple"), "simple");
        assert_eq!(to_camel_case(""), "");
    }

    #[test]
    fn to_camel_case___handles_consecutive_underscores() {
        assert_eq!(to_camel_case("foo__bar"), "fooBar");
        assert_eq!(to_camel_case("_leading"), "Leading");
        assert_eq!(to_camel_case("trailing_"), "trailing");
    }

    #[test]
    fn to_pascal_case___converts_snake_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("display_name"), "DisplayName");
    }

    #[test]
    fn to_pascal_case___converts_kebab_case() {
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("my-plugin"), "MyPlugin");
    }

    #[test]
    fn to_pascal_case___handles_simple_words() {
        assert_eq!(to_pascal_case("hello"), "Hello");
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn capitalize___capitalizes_first_letter() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("world"), "World");
        assert_eq!(capitalize("a"), "A");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn capitalize___preserves_rest_of_string() {
        assert_eq!(capitalize("helloWorld"), "HelloWorld");
        assert_eq!(capitalize("ALLCAPS"), "ALLCAPS");
    }

    #[test]
    fn to_method_name___replaces_dots() {
        assert_eq!(to_method_name("user.create"), "user_create");
        assert_eq!(to_method_name("a.b.c"), "a_b_c");
    }

    #[test]
    fn to_method_name___replaces_dashes() {
        assert_eq!(to_method_name("get-user"), "get_user");
        assert_eq!(to_method_name("a-b-c"), "a_b_c");
    }

    #[test]
    fn to_method_name___handles_mixed() {
        assert_eq!(to_method_name("user.get-by-id"), "user_get_by_id");
    }
}
