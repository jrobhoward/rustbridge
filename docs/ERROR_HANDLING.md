# Error Handling in rustbridge

This guide covers error handling best practices for rustbridge plugins, from Rust implementation through FFI boundaries to host language consumption.

## Table of Contents

1. [Overview](#overview)
2. [Rust Error Types](#rust-error-types)
3. [Error Codes](#error-codes)
4. [Error Propagation](#error-propagation)
5. [Custom Errors](#custom-errors)
6. [FFI Error Handling](#ffi-error-handling)
7. [Java/Kotlin Error Handling](#javakotlin-error-handling)
8. [Testing Error Scenarios](#testing-error-scenarios)
9. [Best Practices](#best-practices)
10. [Common Patterns](#common-patterns)

## Overview

rustbridge uses a structured error handling approach that safely crosses FFI boundaries:

```
Rust Plugin Error → FFI Boundary → Java/Kotlin Exception
   (PluginError)    (error code +    (PluginException)
                      message)
```

**Key principles:**
- Errors never panic across FFI boundaries
- All errors have numeric codes for FFI transmission
- Error messages provide actionable context
- Errors preserve type information where possible

## Rust Error Types

### PluginError Enum

All plugin operations return `PluginResult<T>`, which is an alias for `Result<T, PluginError>`:

```rust
use rustbridge_core::{PluginError, PluginResult};

async fn handle_request(
    &self,
    ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    // Your handler logic
    Ok(response_bytes)
}
```

### Available Error Variants

rustbridge provides these standard error types:

| Variant | Error Code | Use Case |
|---------|-----------|----------|
| `InvalidState` | 1 | Plugin lifecycle state mismatch |
| `InitializationFailed` | 2 | Failed during `on_start()` |
| `ShutdownFailed` | 3 | Failed during `on_stop()` |
| `ConfigError` | 4 | Invalid configuration |
| `SerializationError` | 5 | JSON serialize/deserialize failed |
| `UnknownMessageType` | 6 | Unrecognized type_tag |
| `HandlerError` | 7 | Business logic error |
| `RuntimeError` | 8 | Async runtime error |
| `Cancelled` | 9 | Request was cancelled |
| `Timeout` | 10 | Request timed out |
| `Internal` | 11 | Internal framework error (or panic) |
| `FfiError` | 12 | FFI boundary error |
| `TooManyRequests` | 13 | Concurrency limit exceeded |

## Error Codes

Every `PluginError` maps to a stable numeric code for FFI transmission:

```rust
let error = PluginError::UnknownMessageType("invalid.type".to_string());
let code = error.error_code();  // Returns 6
```

These codes are part of the API contract and **must remain stable** across versions. Never change existing error codes, only add new ones.

## Error Propagation

### The `?` Operator (Recommended)

Use the `?` operator for automatic error propagation:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct MyRequest {
    value: i32,
}

#[derive(Serialize)]
struct MyResponse {
    result: i32,
}

async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    match type_tag {
        "calculate" => {
            // `?` automatically converts serde_json::Error to PluginError
            let req: MyRequest = serde_json::from_slice(payload)?;

            let result = req.value * 2;
            let resp = MyResponse { result };

            // `?` propagates serialization errors too
            Ok(serde_json::to_vec(&resp)?)
        }
        _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
    }
}
```

### Automatic Conversion with `From`

rustbridge automatically converts common errors to `PluginError`:

```rust
impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}
```

This means you can use `?` with any `serde_json::Error` and it will automatically become a `PluginError::SerializationError`.

### Explicit Error Construction

When automatic conversion isn't available, create errors explicitly:

```rust
async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    match type_tag {
        "divide" => {
            let req: DivideRequest = serde_json::from_slice(payload)?;

            if req.divisor == 0 {
                // Explicit error creation
                return Err(PluginError::HandlerError(
                    "Division by zero is not allowed".to_string()
                ));
            }

            let result = req.dividend / req.divisor;
            let resp = DivideResponse { result };
            Ok(serde_json::to_vec(&resp)?)
        }
        _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
    }
}
```

## Custom Errors

For domain-specific errors, you have two options:

### Option 1: Use `HandlerError` (Simple)

Wrap your custom error messages in `HandlerError`:

```rust
#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

async fn handle_create_user(
    &self,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    let req: CreateUserRequest = serde_json::from_slice(payload)?;

    // Validate username length
    if req.username.len() < 3 {
        return Err(PluginError::HandlerError(
            format!("Username must be at least 3 characters, got {}", req.username.len())
        ));
    }

    // Validate email format
    if !req.email.contains('@') {
        return Err(PluginError::HandlerError(
            format!("Invalid email format: {}", req.email)
        ));
    }

    // Create user...
    Ok(response_bytes)
}
```

### Option 2: Custom Error Type with `thiserror` (Advanced)

For more structured errors, define a custom error type:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("username too short: minimum 3 characters, got {0}")]
    UsernameTooShort(usize),

    #[error("invalid email format: {0}")]
    InvalidEmail(String),

    #[error("user already exists: {0}")]
    UserExists(String),

    #[error("database error: {0}")]
    DatabaseError(String),
}

// Convert your error to PluginError
impl From<UserError> for PluginError {
    fn from(err: UserError) -> Self {
        PluginError::HandlerError(err.to_string())
    }
}

// Now you can use ? with UserError
async fn handle_create_user(
    &self,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    let req: CreateUserRequest = serde_json::from_slice(payload)?;

    if req.username.len() < 3 {
        return Err(UserError::UsernameTooShort(req.username.len()).into());
    }

    if !req.email.contains('@') {
        return Err(UserError::InvalidEmail(req.email).into());
    }

    // Database call with automatic error conversion
    save_to_database(&req).map_err(|e| {
        UserError::DatabaseError(e.to_string())
    })?;

    Ok(response_bytes)
}
```

## FFI Error Handling

### How Errors Cross FFI Boundaries

When a Rust error occurs, the FFI layer:

1. **Catches the error** (or panic)
2. **Extracts error code** using `error.error_code()`
3. **Formats error message** using `error.to_string()`
4. **Returns error buffer** to the host language

```rust
// Simplified view of FFI error handling
pub extern "C" fn plugin_call(...) -> FfiBuffer {
    match handle_call_internal(...) {
        Ok(data) => FfiBuffer::success(data),
        Err(error) => FfiBuffer::error(
            error.error_code(),
            error.to_string()
        ),
    }
}
```

### Panic Handling

Panics are caught at the FFI boundary and converted to `PluginError::Internal`:

```rust
// DON'T DO THIS - panics are bad, but they won't crash the host
async fn bad_handler(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    panic!("Something went wrong!");  // Returns error code 11
}

// DO THIS INSTEAD
async fn good_handler(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    Err(PluginError::Internal("Something went wrong!".to_string()))
}
```

While panics won't crash the host application, they do:
- Transition the plugin to `Failed` state
- Log a panic backtrace
- Prevent further requests

**Best practice:** Avoid panics. Use `Result` and proper error handling.

## Java/Kotlin Error Handling

### Java Exception Handling

Errors from Rust become `PluginException` in Java:

```java
import com.rustbridge.Plugin;
import com.rustbridge.PluginException;
import com.rustbridge.ffm.FfmPluginLoader;

public class ErrorHandlingExample {
    public static void main(String[] args) {
        try (Plugin plugin = FfmPluginLoader.load("libmyplugin.so")) {
            try {
                String response = plugin.call("divide", "{\"dividend\": 10, \"divisor\": 0}");
                System.out.println(response);
            } catch (PluginException e) {
                System.err.println("Error: " + e.getMessage());
                System.err.println("Error code: " + e.getErrorCode());

                // Handle specific error codes
                switch (e.getErrorCode()) {
                    case 6:  // UnknownMessageType
                        System.err.println("Invalid operation");
                        break;
                    case 7:  // HandlerError
                        System.err.println("Business logic error");
                        break;
                    case 13:  // TooManyRequests
                        System.err.println("Too busy, retry later");
                        break;
                    default:
                        System.err.println("Unexpected error");
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
```

### Kotlin Exception Handling

Kotlin's when expression makes error handling cleaner:

```kotlin
import com.rustbridge.PluginException
import com.rustbridge.ffm.FfmPluginLoader

fun main() {
    FfmPluginLoader.load("libmyplugin.so").use { plugin ->
        try {
            val response = plugin.call("divide", """{"dividend": 10, "divisor": 0}""")
            println(response)
        } catch (e: PluginException) {
            when (e.errorCode) {
                6 -> println("Invalid operation: ${e.message}")
                7 -> println("Business logic error: ${e.message}")
                13 -> println("Too busy, retry later")
                else -> println("Error (${e.errorCode}): ${e.message}")
            }
        }
    }
}
```

### Typed Error Handling (Kotlin)

For type-safe error handling, wrap calls in a Result type:

```kotlin
sealed class PluginResult<out T> {
    data class Success<T>(val value: T) : PluginResult<T>()
    data class Error(val code: Int, val message: String) : PluginResult<Nothing>()
}

fun <T> Plugin.callSafe(
    messageType: String,
    request: String,
    transform: (String) -> T
): PluginResult<T> {
    return try {
        val response = call(messageType, request)
        PluginResult.Success(transform(response))
    } catch (e: PluginException) {
        PluginResult.Error(e.errorCode, e.message ?: "Unknown error")
    }
}

// Usage
when (val result = plugin.callSafe("divide", request) { it }) {
    is PluginResult.Success -> println("Result: ${result.value}")
    is PluginResult.Error -> println("Error ${result.code}: ${result.message}")
}
```

## Testing Error Scenarios

### Testing Rust Error Paths

Always test error conditions, not just success paths:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rustbridge_core::{PluginConfig, PluginContext};

    #[tokio::test]
    async fn divide___zero_divisor___returns_error() {
        let plugin = CalculatorPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        let request = r#"{"dividend": 10, "divisor": 0}"#;

        let result = plugin
            .handle_request(&ctx, "divide", request.as_bytes())
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            PluginError::HandlerError(msg) => {
                assert!(msg.contains("zero"));
            }
            _ => panic!("Expected HandlerError, got {:?}", error),
        }
    }

    #[tokio::test]
    async fn handle_request___unknown_type___returns_error() {
        let plugin = CalculatorPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        let result = plugin
            .handle_request(&ctx, "unknown.type", b"{}")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::UnknownMessageType(tag) => {
                assert_eq!(tag, "unknown.type");
            }
            _ => panic!("Expected UnknownMessageType"),
        }
    }

    #[tokio::test]
    async fn handle_request___invalid_json___returns_error() {
        let plugin = CalculatorPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        let result = plugin
            .handle_request(&ctx, "divide", b"not json")
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }
}
```

### Testing Java/Kotlin Error Handling

Test that errors propagate correctly across FFI:

```java
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class ErrorHandlingTest {
    @Test
    void call___division_by_zero___throws_exception() {
        try (Plugin plugin = FfmPluginLoader.load("libcalculator.so")) {
            PluginException exception = assertThrows(
                PluginException.class,
                () -> plugin.call("divide", "{\"dividend\": 10, \"divisor\": 0}")
            );

            assertEquals(7, exception.getErrorCode());  // HandlerError
            assertTrue(exception.getMessage().contains("zero"));
        }
    }

    @Test
    void call___unknown_message_type___throws_exception() {
        try (Plugin plugin = FfmPluginLoader.load("libcalculator.so")) {
            PluginException exception = assertThrows(
                PluginException.class,
                () -> plugin.call("unknown.type", "{}")
            );

            assertEquals(6, exception.getErrorCode());  // UnknownMessageType
            assertTrue(exception.getMessage().contains("unknown"));
        }
    }

    @Test
    void call___exceeds_concurrency_limit___throws_exception() throws Exception {
        PluginConfig config = PluginConfig.defaults().maxConcurrentOps(1);

        try (Plugin plugin = FfmPluginLoader.load("libcalculator.so", config)) {
            // Implementation depends on your concurrency testing strategy
            // See ConcurrencyLimitTest.java for comprehensive examples
        }
    }
}
```

## Best Practices

### 1. Never Use `.unwrap()` in Production Code

```rust
// ❌ BAD - will panic
let config: MyConfig = serde_json::from_slice(payload).unwrap();

// ✅ GOOD - propagates error
let config: MyConfig = serde_json::from_slice(payload)?;
```

### 2. Provide Actionable Error Messages

```rust
// ❌ BAD - not actionable
return Err(PluginError::HandlerError("Invalid input".to_string()));

// ✅ GOOD - tells user what's wrong and how to fix it
return Err(PluginError::HandlerError(
    format!("Username must be 3-20 characters, got {}", username.len())
));
```

### 3. Use Specific Error Variants

```rust
// ❌ BAD - loses type information
return Err(PluginError::Internal("Config missing".to_string()));

// ✅ GOOD - uses appropriate error type
return Err(PluginError::ConfigError("Required field 'api_key' is missing".to_string()));
```

### 4. Log Errors with Context

```rust
async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    let req: MyRequest = serde_json::from_slice(payload)
        .map_err(|e| {
            tracing::error!("Failed to deserialize {}: {}", type_tag, e);
            PluginError::SerializationError(e.to_string())
        })?;

    // ... rest of handler
}
```

### 5. Don't Swallow Errors

```rust
// ❌ BAD - loses error information
let result = dangerous_operation();
if result.is_err() {
    return Err(PluginError::Internal("Operation failed".to_string()));
}

// ✅ GOOD - preserves error details
let result = dangerous_operation()
    .map_err(|e| {
        PluginError::Internal(format!("Dangerous operation failed: {}", e))
    })?;
```

### 6. Handle Errors at the Right Level

```rust
// ✅ GOOD - handle errors where you have context
async fn save_user(&self, user: &User) -> Result<(), DatabaseError> {
    // Database-specific error handling
    self.db.insert(user).await
        .map_err(|e| DatabaseError::InsertFailed(e.to_string()))
}

async fn handle_create_user(
    &self,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    let req: CreateUserRequest = serde_json::from_slice(payload)?;
    let user = User::from_request(&req);

    // Convert database error to plugin error
    self.save_user(&user).await
        .map_err(|e| PluginError::HandlerError(e.to_string()))?;

    Ok(serde_json::to_vec(&CreateUserResponse { ... })?)
}
```

## Common Patterns

### Validation Pattern

```rust
fn validate_username(username: &str) -> PluginResult<()> {
    if username.len() < 3 {
        return Err(PluginError::HandlerError(
            format!("Username too short: {} chars", username.len())
        ));
    }
    if username.len() > 20 {
        return Err(PluginError::HandlerError(
            format!("Username too long: {} chars", username.len())
        ));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(PluginError::HandlerError(
            "Username must be alphanumeric or underscore".to_string()
        ));
    }
    Ok(())
}

async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    match type_tag {
        "user.create" => {
            let req: CreateUserRequest = serde_json::from_slice(payload)?;
            validate_username(&req.username)?;  // Validate before processing
            // ... process request
        }
        _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
    }
}
```

### Retry Pattern (Transient Errors)

```rust
async fn call_external_api(&self, request: &ApiRequest) -> Result<ApiResponse, ApiError> {
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 0..max_retries {
        match self.http_client.post("/api/endpoint", request).await {
            Ok(response) => return Ok(response),
            Err(e) if e.is_transient() => {
                tracing::warn!("Transient error on attempt {}: {}", attempt + 1, e);
                last_error = Some(e);
                tokio::time::sleep(Duration::from_millis(100 * (attempt + 1))).await;
            }
            Err(e) => return Err(e),  // Non-transient error, fail immediately
        }
    }

    Err(last_error.unwrap_or_else(|| ApiError::Unknown))
}
```

### Fallback Pattern

```rust
async fn get_config_value(&self, key: &str) -> PluginResult<String> {
    // Try cache first
    if let Some(value) = self.cache.get(key) {
        return Ok(value);
    }

    // Fallback to database
    match self.db.get(key).await {
        Ok(value) => {
            self.cache.insert(key, &value);
            Ok(value)
        }
        Err(e) => {
            tracing::warn!("Failed to get config from DB: {}", e);
            // Fallback to default
            Ok(self.default_config.get(key).unwrap_or("default").to_string())
        }
    }
}
```

## Summary

**Key takeaways:**
- ✅ Use `PluginResult<T>` for all fallible operations
- ✅ Use `?` operator for error propagation
- ✅ Provide actionable error messages with context
- ✅ Test error paths, not just success paths
- ✅ Never panic in production code
- ✅ Use appropriate error variants for different failure modes
- ✅ Handle errors at the right level with proper context
- ✅ Log errors with sufficient detail for debugging

For more information:
- [docs/SKILLS.md](./SKILLS.md) - Development best practices
- [docs/ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [docs/TESTING.md](./TESTING.md) - Testing conventions
