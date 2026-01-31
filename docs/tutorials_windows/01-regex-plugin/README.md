# Chapter 1: Building a Regex Plugin

In this chapter, you'll build your first rustbridge plugin—a regex engine that compiles patterns once and reuses them efficiently using an LRU cache.

## What You'll Build

A regex plugin that:
- Compiles regex patterns on first use
- Caches compiled patterns for reuse (LRU eviction)
- Supports configurable cache sizes
- Reports match results as structured JSON

## Prerequisites

- Rust 1.90+ with the Windows MSVC toolchain
- Visual Studio Build Tools with C++ development workload
- The rustbridge CLI (`cargo install --git https://github.com/example/rustbridge rustbridge-cli`)

## Sections

### [01: Project Scaffold](./01-scaffold.md)
Set up the project structure using `rustbridge new`.

### [02: Basic Matching](./02-basic-matching.md)
Implement the core regex matching functionality.

### [03: LRU Cache](./03-lru-cache.md)
Add an LRU cache to avoid recompiling patterns.

### [04: Configuration](./04-configuration.md)
Make the cache size configurable from the host.

## Time Estimate

This chapter takes approximately 30-45 minutes to complete.

## What You'll Learn

- Using `rustbridge new` to scaffold a plugin project
- Implementing the `Plugin` trait
- Defining message types with serde
- Using async/await in plugin handlers
- Adding external crate dependencies
- Testing plugins locally

## Next Steps

After completing this chapter, continue to [Chapter 2: Kotlin Consumer](../02-kotlin-consumer/README.md) to call your plugin from Kotlin.
