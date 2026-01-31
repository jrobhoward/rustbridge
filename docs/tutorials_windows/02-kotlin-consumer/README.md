# Chapter 2: Kotlin Consumer

In this chapter, you'll call your regex plugin from Kotlin using Java's Foreign Function & Memory (FFM) API.

## What You'll Build

A Kotlin application that:
- Loads the regex plugin from a bundle
- Calls the match and find_all endpoints
- Receives structured log callbacks
- Uses type-safe wrapper classes

## Prerequisites

- Completed [Chapter 1: Regex Plugin](../01-regex-plugin/README.md)
- JDK 21+ (for FFM API)
- Gradle 8.5+

## Sections

### [01: Project Setup](./01-project-setup.md)
Create a Gradle project with rustbridge dependencies.

### [02: Calling the Plugin](./02-calling-plugin.md)
Load the plugin and make JSON calls.

### [03: Logging Callbacks](./03-logging-callbacks.md)
Receive log messages from the plugin.

### [04: Type-Safe Calls](./04-type-safe-calls.md)
Create wrapper classes for type-safe message handling.

### [05: Benchmarking](./05-benchmarking.md)
Measure cache hit performance improvements.

## What You'll Learn

- Setting up a Kotlin project with rustbridge
- Using the FFM API for native library calls
- JSON serialization with kotlinx.serialization
- Callback patterns for logging
- Creating idiomatic Kotlin wrappers

## Next Steps

After completing this chapter, continue to [Chapter 3: JSON Plugin](../03-json-plugin/README.md) to build a more complex plugin.
