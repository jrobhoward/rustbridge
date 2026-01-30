# Chapter 3: Building a JSON Plugin

In this chapter, you'll build a Rust plugin that validates and pretty-prints JSON strings. This builds on the concepts from Chapter 1 while introducing multiple message types in a single plugin.

## What You'll Build

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          JSON Plugin Architecture                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Request: { "json": "{\"name\":\"test\"}" }                                 │
│                       │                                                     │
│                       ▼                                                     │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  Message Type: "validate"              Message Type: "prettify"      │   │
│  │  ─────────────────────────             ────────────────────────      │   │
│  │  1. Parse JSON string                  1. Parse JSON string          │   │
│  │  2. Return { valid: true/false }       2. Pretty-print with indent   │   │
│  │                                        3. Return { result: "..." }   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Response: { "valid": true }              Response: { "result": "{\n..." }  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Sections

### [01: Scaffold the Project](./01-scaffold.md)
Generate a new plugin project with Java consumer support.

### [02: Validate Message](./02-validate-message.md)
Add a message type that checks if a string contains valid JSON.

### [03: Prettify Message](./03-prettify-message.md)
Add a second message type that formats JSON with indentation.

### [04: Error Handling](./04-error-handling.md)
Return meaningful errors when JSON is invalid.

## Prerequisites

- Completed Chapter 1 (regex plugin) or familiarity with rustbridge basics
- Rust 1.90+ installed
- rustbridge CLI installed

## Time Estimate

This chapter takes approximately 20-30 minutes to complete.

## Reference Implementation

The completed plugin is available at [`examples/json-plugin/`](../../../examples/json-plugin/) for reference.

## Next Steps

After completing this chapter, continue to [Chapter 4: Calling from Java](../04-java-consumer/README.md) to call your plugin from a Java application.
