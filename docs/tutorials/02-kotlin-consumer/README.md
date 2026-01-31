# Chapter 2: Calling from Kotlin

In this chapter, you'll load your regex plugin from Kotlin, make type-safe calls, and capture plugin logs.

## What You'll Build

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Kotlin Consumer Architecture                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Kotlin Application                                                         │
│  ─────────────────────                                                      │
│  data class MatchRequest(val pattern: String, val text: String)             │
│  data class MatchResponse(val matches: Boolean, val cached: Boolean)        │
│                                                                             │
│  val response = plugin.callTyped<MatchResponse>("match", request)           │
│                       │                                                     │
│                       ▼                                                     │
│  ┌──────────────────────────────────────────────────────┐                   │
│  │  BundleLoader.extractLibrary()                       │                   │
│  │  FfmPluginLoader.load(library)                       │                   │
│  │  plugin.setLogCallback { level, msg -> ... }         │                   │
│  │  plugin.call("match", json) → json                   │                   │
│  └──────────────────────────────────────────────────────┘                   │
│                       │                                                     │
│                       ▼                                                     │
│              ┌─────────────────┐                                            │
│              │  regex-plugin   │                                            │
│              │  (.rbp bundle)  │                                            │
│              └─────────────────┘                                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Sections

### [01: Project Setup](./01-project-setup.md)
Copy the Kotlin template, configure Gradle, and build your plugin bundle.

### [02: Calling the Plugin](./02-calling-plugin.md)
Load the bundle, initialize the plugin, and make JSON calls.

### [03: Logging Callbacks](./03-logging-callbacks.md)
Capture plugin logs in your Kotlin application.

### [04: Type-Safe Calls](./04-type-safe-calls.md)
Define data classes and use extension functions for type safety.

### [05: Benchmarking](./05-benchmarking.md)
Compare debug vs release builds and measure cache effectiveness.

## Prerequisites

- Java 21+ installed (for FFM support)
- Your regex plugin bundle from Chapter 1
- Basic familiarity with Kotlin/Gradle

## Time Estimate

This chapter takes approximately 20-30 minutes to complete.

## Next Steps

After completing this chapter, continue to [Chapter 3: Building a JSON Plugin](../03-json-plugin/README.md) to learn more message handling patterns.
