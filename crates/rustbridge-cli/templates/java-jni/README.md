# rustbridge Java JNI Consumer Template (Recommended)

A minimal Java 17+ project template for consuming rustbridge plugins using JNI.

This is the **recommended** approach for all Java versions (17+). JNI provides excellent compatibility across all LTS releases (Java 17, 21, 25+) and has the best binary transport performance.

## Prerequisites

- **Java 17+** - JNI works on all Java 17+ versions (LTS: 17, 21, 25)
- **Gradle 8.0+** - Build tool
- **A rustbridge plugin** - Your `.rbp` bundle file
- **JNI bridge library** - Built from rustbridge

## Quick Start

1. **Copy this template** to your project location
2. **Build the JNI bridge**:
   ```bash
   cd /path/to/rustbridge
   cargo build --release -p rustbridge-jni
   ```
3. **Install rustbridge Java libraries**:
   ```bash
   cd /path/to/rustbridge/rustbridge-java
   ./gradlew publishToMavenLocal
   ```
4. **Update java.library.path** in `build.gradle.kts` to point to where `librustbridge_jni.so` is located
5. **Add your plugin bundle** - Copy your `.rbp` file to the project root
6. **Update Main.java** - Set `bundlePath` to your `.rbp` file
7. **Run**:
   ```bash
   ./gradlew run
   ```

## Library Path

JNI requires the native library to be in `java.library.path`. You can set this via:

```bash
java -Djava.library.path=/path/to/rustbridge/target/release -jar myapp.jar
```

Or set `LD_LIBRARY_PATH` (Linux) / `DYLD_LIBRARY_PATH` (macOS):

```bash
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/rustbridge/target/release
```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [Java JNI Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/JAVA_JNI.md)
