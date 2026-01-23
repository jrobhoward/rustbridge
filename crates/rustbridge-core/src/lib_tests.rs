#![allow(non_snake_case)]

use super::*;

// LogLevel tests

#[test]
fn LogLevel___ordering___trace_less_than_debug() {
    assert!(LogLevel::Trace < LogLevel::Debug);
}

#[test]
fn LogLevel___ordering___debug_less_than_info() {
    assert!(LogLevel::Debug < LogLevel::Info);
}

#[test]
fn LogLevel___ordering___info_less_than_warn() {
    assert!(LogLevel::Info < LogLevel::Warn);
}

#[test]
fn LogLevel___ordering___warn_less_than_error() {
    assert!(LogLevel::Warn < LogLevel::Error);
}

#[test]
fn LogLevel___ordering___error_less_than_off() {
    assert!(LogLevel::Error < LogLevel::Off);
}

#[test]
fn LogLevel___from_u8___0_returns_trace() {
    assert_eq!(LogLevel::from_u8(0), LogLevel::Trace);
}

#[test]
fn LogLevel___from_u8___1_returns_debug() {
    assert_eq!(LogLevel::from_u8(1), LogLevel::Debug);
}

#[test]
fn LogLevel___from_u8___2_returns_info() {
    assert_eq!(LogLevel::from_u8(2), LogLevel::Info);
}

#[test]
fn LogLevel___from_u8___3_returns_warn() {
    assert_eq!(LogLevel::from_u8(3), LogLevel::Warn);
}

#[test]
fn LogLevel___from_u8___4_returns_error() {
    assert_eq!(LogLevel::from_u8(4), LogLevel::Error);
}

#[test]
fn LogLevel___from_u8___5_returns_off() {
    assert_eq!(LogLevel::from_u8(5), LogLevel::Off);
}

#[test]
fn LogLevel___from_u8___invalid_returns_off() {
    assert_eq!(LogLevel::from_u8(100), LogLevel::Off);
}

#[test]
fn LogLevel___display___trace_shows_uppercase() {
    assert_eq!(LogLevel::Trace.to_string(), "TRACE");
}

#[test]
fn LogLevel___display___info_shows_uppercase() {
    assert_eq!(LogLevel::Info.to_string(), "INFO");
}

#[test]
fn LogLevel___display___error_shows_uppercase() {
    assert_eq!(LogLevel::Error.to_string(), "ERROR");
}
