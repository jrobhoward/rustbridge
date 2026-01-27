# rustbridge Kotlin Consumer Template

A minimal Kotlin project template for consuming rustbridge plugins.

## Prerequisites

- **Java 21+** - Required for FFM (Foreign Function & Memory API)
- **Gradle 8.0+** - Build tool
- **A rustbridge plugin** - Your `.rbp` bundle file

## Quick Start

1. **Copy this template** to your project location (from the rustbridge repo):
   ```bash
   cp -r templates/kotlin ~/my-kotlin-app
   cd ~/my-kotlin-app
   ```

2. **Install rustbridge Java libraries** (if not published to Maven Central):
   ```bash
   cd /path/to/rustbridge/rustbridge-java
   ./gradlew publishToMavenLocal
   ```

3. **Add your plugin bundle** - Copy your `.rbp` file to the project root

4. **Update Main.kt** - Edit `src/main/kotlin/com/example/Main.kt`:
   - Set `bundlePath` to your `.rbp` file
   - Define request/response data classes matching your plugin's API

5. **Run**:
   ```bash
   ./gradlew run
   ```

## Project Structure

```
├── build.gradle.kts          # Gradle build configuration
├── settings.gradle.kts       # Project settings
├── gradle.properties         # Gradle properties
└── src/main/kotlin/
    └── com/example/
        └── Main.kt           # Your application entry point
```

## Configuration

### JVM Arguments

The `build.gradle.kts` includes required JVM arguments for FFM:

```kotlin
tasks.withType<JavaExec> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
```

### Dependencies

- `rustbridge-core` - Core interfaces and types
- `rustbridge-ffm` - FFM-based native plugin loader
- `jackson-module-kotlin` - JSON serialization

## Type-Safe Calls

The template includes a `callTyped` extension function for type-safe plugin calls:

```kotlin
data class MyRequest(val input: String)
data class MyResponse(val output: String)

val response = plugin.callTyped<MyResponse>("my.message.type", MyRequest("hello"))
```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [Kotlin Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/KOTLIN.md)

## License

MIT
