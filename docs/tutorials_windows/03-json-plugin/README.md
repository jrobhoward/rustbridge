# Chapter 3: Building a JSON Plugin

In this chapter, you'll build a JSON toolkit plugin that provides validation and formatting capabilities.

## What You'll Build

A JSON plugin that:
- Validates JSON strings and reports errors
- Prettifies JSON with configurable indentation
- Minifies JSON by removing whitespace
- Reports detailed parse error locations

## Prerequisites

- Completed [Chapter 1: Regex Plugin](../01-regex-plugin/README.md)
- Rust 1.90+ with Windows MSVC toolchain

## Sections

### [01: Project Scaffold](./01-scaffold.md)
Create the json-plugin project structure.

### [02: Validate Message](./02-validate-message.md)
Implement JSON validation with detailed error reporting.

### [03: Prettify Message](./03-prettify-message.md)
Implement JSON formatting with configurable indentation.

### [04: Error Handling](./04-error-handling.md)
Implement proper error types and handling.

## What You'll Learn

- Handling multiple message types
- Structured error responses
- Using serde_json for validation
- Error position reporting

## Next Steps

After completing this chapter, continue to [Chapter 4: Java Consumer](../04-java-consumer/README.md) to call this plugin from Java.
