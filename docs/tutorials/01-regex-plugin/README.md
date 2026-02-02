# Chapter 1: Building a Regex Plugin

In this chapter, you'll build a Rust plugin that matches text against regex patterns, with an LRU cache for compiled patterns and configurable cache size.

## What You'll Build

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Regex Plugin Architecture                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Request: { pattern: "\\d+", text: "test123" }                              │
│                       │                                                     │
│                       ▼                                                     │
│  ┌──────────────────────────────────────────────────────┐                   │
│  │  1. Check LRU Cache for pattern                      │                   │
│  │     ├─ Hit: Use cached Regex                         │                   │
│  │     └─ Miss: Compile & cache                         │                   │
│  │  2. Run regex.is_match(text)                         │                   │
│  │  3. Return { matches: true, cached: false }          │                   │
│  └──────────────────────────────────────────────────────┘                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Sections

### [01: Scaffold the Project](./01-scaffold.md)
Generate a new plugin project using `rustbridge new` and verify it builds.

### [02: Basic Regex Matching](./02-basic-matching.md)
Define the message types and implement pattern matching.

### [03: Add LRU Caching](./03-lru-cache.md)
Cache compiled regex patterns and measure the performance improvement.

### [04: Make It Configurable](./04-configuration.md)
Allow the host to configure cache size at initialization.

## Prerequisites

- Rust 1.90+ installed
- rustbridge CLI installed (`cargo install --path crates/rustbridge-cli`)
- Basic familiarity with Rust

## Time Estimate

This chapter takes approximately 30-45 minutes to complete.

## Next Steps

After completing this chapter, continue to [Chapter 2: Calling from Kotlin](../02-kotlin-consumer/README.md).
