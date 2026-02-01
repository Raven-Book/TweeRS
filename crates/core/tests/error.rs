// Unit tests for error handling
use std::io;
use tweers_core::error::TweersError;

#[test]
fn test_error_from_io() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let tweers_err: TweersError = io_err.into();

    assert!(matches!(tweers_err, TweersError::Io(_)));
    assert!(tweers_err.to_string().contains("I/O error"));
}

#[test]
fn test_error_parse() {
    let err = TweersError::parse("invalid syntax");
    assert!(matches!(err, TweersError::Parse(_)));
    assert_eq!(err.to_string(), "Parse error: invalid syntax");
}

#[test]
fn test_error_missing_input() {
    let err = TweersError::missing_input("sources");
    assert!(matches!(err, TweersError::MissingInput(_)));
    assert_eq!(err.to_string(), "Missing required input: sources");
}
