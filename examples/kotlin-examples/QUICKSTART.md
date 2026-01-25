# Quick Start Guide

Get up and running with rustbridge Kotlin examples in 5 minutes.

## Prerequisites

1. **Java 21+**
   ```bash
   java -version  # Should show Java 21 or higher
   ```

2. **Rust toolchain** (to build the plugin)
   ```bash
   rustc --version  # Should show Rust 1.70 or higher
   ```

## Step 1: Build the Plugin

```bash
cd ../hello-plugin
cargo build --release

# Verify the library was built
ls -lh target/release/libhello_plugin.so  # Linux
ls -lh target/release/libhello_plugin.dylib  # macOS
ls -lh target/release/hello_plugin.dll  # Windows
```

## Step 2: Build rustbridge Java Libraries

```bash
cd ../../rustbridge-java

# Build all Java modules
./gradlew build

# This creates:
# - rustbridge-core/build/libs/rustbridge-core.jar
# - rustbridge-ffm/build/libs/rustbridge-ffm.jar
```

## Step 3: Run Kotlin Examples

```bash
cd ../examples/kotlin-examples

# Run all examples
./gradlew run

# Or run specific examples
./gradlew runBasic
./gradlew runLogging
./gradlew runErrorHandling
```

## Expected Output

### Basic Example

```
=== rustbridge Kotlin Basic Example ===

Loading plugin from: /path/to/libhello_plugin.so

1. Echo Example:
   Response: Hello from Kotlin!
   Length: 19

2. Greet Example:
   Hello, Kotlin Developer! Welcome to rustbridge.

3. Create User Example:
   User ID: user-00000000
   Created at: 1706036400Z

4. Math Add Example:
   42 + 58 = 100

5. Multiple User Creation:
   Created user-00000001
   Created user-00000002
   Created user-00000003

=== Example Complete ===
```

## Troubleshooting

### "Could not find libhello_plugin.so"

Make sure you built the plugin:

```bash
cd ../hello-plugin
cargo build --release
```

### "Could not find rustbridge-core.jar"

Build the Java libraries:

```bash
cd ../../rustbridge-java
./gradlew build
```

### "Unsupported class file major version"

You need Java 21+:

```bash
# Install Java 21 (Ubuntu/Debian)
sudo apt install openjdk-21-jdk

# Install Java 21 (macOS with Homebrew)
brew install openjdk@21

# Set JAVA_HOME
export JAVA_HOME=/path/to/java-21
```

### UnsatisfiedLinkError on macOS

If you get library loading errors on macOS:

```bash
# Clear quarantine attribute
xattr -d com.apple.quarantine ../hello-plugin/target/release/libhello_plugin.dylib
```

## Next Steps

1. **Read the examples**: Check out the `.kt` files in `src/main/kotlin/`
2. **Try modifications**: Edit the examples to experiment
3. **Compare with Java**: Read `COMPARISON.md` to see Java vs Kotlin
4. **Build your own**: Use the examples as templates for your own plugins

## IDE Setup

### IntelliJ IDEA (Recommended)

1. Open the `kotlin-examples` directory
2. IDEA will auto-detect Gradle and import the project
3. Make sure Project SDK is set to Java 21+
4. Run examples directly from the IDE

### VS Code

1. Install the "Kotlin" extension
2. Install the "Gradle for Java" extension
3. Open the `kotlin-examples` directory
4. Use the Gradle sidebar to run tasks

## Performance Tips

- **Release builds**: Always use `cargo build --release` for the plugin
- **Worker threads**: Adjust `workerThreads` in PluginConfig based on your workload
- **Log level**: Use "warn" or "error" in production for better performance
- **Reuse plugin**: Create the plugin once and reuse it for multiple calls

## Learn More

- [README.md](./README.md) - Overview of all examples
- [COMPARISON.md](./COMPARISON.md) - Java vs Kotlin comparison
- [../../docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - rustbridge architecture
- [../../README.md](../../README.md) - Main project documentation
