#![allow(non_snake_case)]

use super::*;

// RequestContext tests

#[test]
fn RequestContext___new___sets_request_id_and_type_tag() {
    let ctx = RequestContext::new(123, "user.create");

    assert_eq!(ctx.request_id, 123);
    assert_eq!(ctx.type_tag, "user.create");
    assert!(ctx.correlation_id.is_none());
}

#[test]
fn RequestContext___with_correlation_id___sets_correlation_id() {
    let ctx = RequestContext::new(123, "user.create").with_correlation_id("corr-456");

    assert_eq!(ctx.correlation_id, Some("corr-456".to_string()));
}

// ResponseBuilder tests

#[test]
fn ResponseBuilder___new___creates_empty_builder() {
    let builder = ResponseBuilder::new();

    let result = builder.build();

    assert!(result.is_success());
    assert_eq!(result.data(), Some([].as_slice()));
}

#[test]
fn ResponseBuilder___data___sets_response_data() {
    let result = ResponseBuilder::new().data(b"hello".to_vec()).build();

    assert!(result.is_success());
    assert_eq!(result.data(), Some(b"hello".as_slice()));
}

#[test]
fn ResponseBuilder___json___serializes_value() {
    #[derive(Serialize)]
    struct TestData {
        value: i32,
    }

    let result = ResponseBuilder::new()
        .json(&TestData { value: 42 })
        .unwrap()
        .build();

    assert!(result.is_success());
    assert!(result.data().is_some());
}

#[test]
fn ResponseBuilder___error___creates_error_response() {
    let result = ResponseBuilder::new().error(404, "Not found").build();

    assert!(!result.is_success());
    let err = result.error().unwrap();
    assert_eq!(err.code, 404);
    assert_eq!(err.message, "Not found");
}

#[test]
fn ResponseBuilder___error_with_details___includes_details() {
    let details = serde_json::json!({"field": "email"});

    let result = ResponseBuilder::new()
        .error_with_details(400, "Validation failed", details)
        .build();

    assert!(!result.is_success());
    let err = result.error().unwrap();
    assert_eq!(err.code, 400);
    assert!(err.details.is_some());
}

#[test]
fn ResponseBuilder___error_overrides_data___when_both_set() {
    let result = ResponseBuilder::new()
        .data(b"ignored".to_vec())
        .error(500, "Error wins")
        .build();

    assert!(!result.is_success());
}

// ResponseResult tests

#[test]
fn ResponseResult___is_success___true_for_success() {
    let result = ResponseResult::Success(vec![1, 2, 3]);

    assert!(result.is_success());
}

#[test]
fn ResponseResult___is_success___false_for_error() {
    let result = ResponseResult::Error(ResponseError {
        code: 500,
        message: "Error".to_string(),
        details: None,
    });

    assert!(!result.is_success());
}

#[test]
fn ResponseResult___data___returns_some_for_success() {
    let result = ResponseResult::Success(vec![1, 2, 3]);

    assert_eq!(result.data(), Some([1u8, 2, 3].as_slice()));
}

#[test]
fn ResponseResult___data___returns_none_for_error() {
    let result = ResponseResult::Error(ResponseError {
        code: 500,
        message: "Error".to_string(),
        details: None,
    });

    assert!(result.data().is_none());
}

#[test]
fn ResponseResult___error___returns_none_for_success() {
    let result = ResponseResult::Success(vec![]);

    assert!(result.error().is_none());
}

#[test]
fn ResponseResult___error___returns_some_for_error() {
    let result = ResponseResult::Error(ResponseError {
        code: 404,
        message: "Not found".to_string(),
        details: None,
    });

    let err = result.error().unwrap();
    assert_eq!(err.code, 404);
}
