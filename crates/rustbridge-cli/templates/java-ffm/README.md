# rustbridge Java FFM Consumer Template (Experimental)

A minimal Java 22+ project template for consuming rustbridge plugins using FFM (Foreign Function & Memory API).

> **Note:** FFM is experimental. For most use cases, we recommend the [JNI template](../java-jni/) which works on Java 17+ and has better binary transport performance.

## Prerequisites

- **Java 22+** - Required for FFM (final API since Java 22)
- **Gradle 8.0+** - Build tool
- **A rustbridge plugin** - Your `.rbp` bundle file

## Quick Start

1. **Copy this template** to your project location
2. **Install rustbridge Java libraries** (if not published to Maven Central):
   ```bash
   cd /path/to/rustbridge/rustbridge-java
   ./gradlew publishToMavenLocal
   ```
3. **Add your plugin bundle** - Copy your `.rbp` file to the project root
4. **Update Main.java** - Set `bundlePath` to your `.rbp` file
5. **Run**:
   ```bash
   ./gradlew run
   ```

## JVM Arguments

FFM requires this JVM flag (already configured in `build.gradle.kts`):

```
--enable-native-access=ALL-UNNAMED
```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [Java FFM Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/JAVA_FFM.md)
