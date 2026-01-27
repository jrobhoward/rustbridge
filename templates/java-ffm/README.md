# rustbridge Java FFM Consumer Template

A minimal Java 21+ project template for consuming rustbridge plugins using FFM (Foreign Function & Memory API).

## Prerequisites

- **Java 21+** - Required for FFM
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

FFM requires these JVM flags (already configured in `build.gradle.kts`):

```
--enable-preview --enable-native-access=ALL-UNNAMED
```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [Java FFM Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/JAVA_FFM.md)
