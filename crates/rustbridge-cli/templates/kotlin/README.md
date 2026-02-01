# rustbridge Kotlin Consumer Template

A minimal Kotlin project template for consuming rustbridge plugins using JNI.

## Prerequisites

- **Java 17+** - JNI works with Java 17+ (LTS versions: 17, 21, 25)
- **Gradle 8.0+** - Build tool
- **A rustbridge plugin** - Your `.rbp` bundle file
- **JNI bridge library** - `librustbridge_jni.so` (built from rustbridge)

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

3. **Build the JNI bridge** (if not already built):
   ```bash
   cd /path/to/rustbridge
   cargo build --release -p rustbridge-jni
   ```

4. **Add your plugin bundle** - Copy your `.rbp` file to the project root

5. **Update Main.kt** - Edit `src/main/kotlin/com/example/Main.kt`:
   - Set `bundlePath` to your `.rbp` file
   - Define request/response data classes matching your plugin's API

6. **Run**:
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

### JNI Library Path

The `build.gradle.kts` configures the JNI native library path:

```kotlin
tasks.withType<JavaExec> {
    systemProperty("java.library.path", System.getProperty("java.library.path", "") +
        ":../../target/release")
}
```

Update the path to point to where `librustbridge_jni.so` is located.

### Dependencies

- `rustbridge-core` - Core interfaces and types
- `rustbridge-jni` - JNI-based native plugin loader (Java 17+)
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
