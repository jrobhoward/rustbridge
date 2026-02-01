# Chapter 4: Calling from Java

In this chapter, you'll load your JSON plugin from Java using the Foreign Function & Memory (FFM) API available in Java
22+ (FFM became a final API in Java 22).

## What You'll Build

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Java Consumer Architecture                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Java Application                                                           │
│  ─────────────────                                                          │
│  record ValidateRequest(String json) {}                                     │
│  record ValidateResponse(boolean valid) {}                                  │
│                                                                             │
│  String response = plugin.call("validate", requestJson);                    │
│                       │                                                     │
│                       ▼                                                     │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  BundleLoader.extractLibrary()                                       │   │
│  │  FfmPluginLoader.load(library)                                       │   │
│  │  plugin.call("validate", json) → json                                │   │
│  │  plugin.call("prettify", json) → json                                │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                       │                                                     │
│                       ▼                                                     │
│              ┌─────────────────┐                                            │
│              │  json-plugin    │                                            │
│              │  (.rbp bundle)  │                                            │
│              └─────────────────┘                                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Sections

### [01: Project Setup](./01-project-setup.md)

Use the generated Java FFM consumer project and configure it.

### [02: Calling the Plugin](./02-calling-plugin.md)

Load the bundle and make type-safe calls with Java records and Gson.

### [03: Error Handling](./03-error-handling.md)

Handle plugin errors gracefully in Java.

## Prerequisites

- **Java 22+** installed
- Your json-plugin bundle from Chapter 3
- Basic familiarity with Java and Gradle

## Time Estimate

This chapter takes approximately 20-30 minutes to complete.

## JNI Alternative

If you prefer JNI (works on Java 17+), see the [Java JNI appendix](../appendix-java-jni/README.md) for an alternative approach.
For Java 22+, we recommend FFM as shown in this chapter.

## Next Steps

After completing this chapter, continue to [Chapter 5: Production Bundles](../05-production-bundles/README.md) to learn
about signing, schemas, and other bundle features.
