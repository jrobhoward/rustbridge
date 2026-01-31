# Chapter 4: Java Consumer

In this chapter, you'll call the JSON plugin from Java using both FFM (Java 21+) and JNI (Java 17+) approaches.

## What You'll Build

A Java application that:
- Loads the JSON plugin from a bundle
- Validates, prettifies, and minifies JSON
- Handles errors gracefully

## Prerequisites

- Completed [Chapter 3: JSON Plugin](../03-json-plugin/README.md)
- JDK 17+ (for JNI) or JDK 21+ (for FFM)
- Gradle 8.5+

## Sections

### [01: Project Setup](./01-project-setup.md)
Create a Gradle project with rustbridge dependencies.

### [02: Calling the Plugin](./02-calling-plugin.md)
Load the plugin and make JSON calls.

### [03: Error Handling](./03-error-handling.md)
Handle validation errors and plugin exceptions.

## What You'll Learn

- Setting up a Java project with rustbridge
- Choosing between FFM and JNI
- JSON serialization with Gson
- Exception handling patterns

## Next Steps

After completing this chapter, continue to [Chapter 5: Production Bundles](../05-production-bundles/README.md) to prepare your plugin for production.
