# Getting Started: Java 8+ (JNI)

This guide walks you through using rustbridge plugins from Java 8+ using Java Native Interface (JNI). Use this for compatibility with older Java versions; prefer FFM for Java 21+.

## Prerequisites

- **Java 8 or later** - JNI works with any Java version
  ```bash
  java -version  # Should be >= 8
  ```
- **Gradle or Maven** - For dependency management
- **A rustbridge plugin** - Either a `.rbp` bundle or `.so`/`.dylib`/`.dll` file
- **rustbridge-jni native library** - Built from `cargo build -p rustbridge-jni`

## Add Dependencies

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("com.rustbridge:rustbridge-core:0.1.0")
    implementation("com.rustbridge:rustbridge-jni:0.1.0")
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'com.rustbridge:rustbridge-core:0.1.0'
    implementation 'com.rustbridge:rustbridge-jni:0.1.0'
}
```

### Maven

```xml
<dependencies>
    <dependency>
        <groupId>com.rustbridge</groupId>
        <artifactId>rustbridge-core</artifactId>
        <version>0.1.0</version>
    </dependency>
    <dependency>
        <groupId>com.rustbridge</groupId>
        <artifactId>rustbridge-jni</artifactId>
        <version>0.1.0</version>
    </dependency>
</dependencies>
```

## Build the JNI Bridge

The JNI implementation requires a native library that bridges Java to the plugin:

```bash
# Build the JNI native library
cargo build --release -p rustbridge-jni
```

This produces:
- Linux: `target/release/librustbridge_jni.so`
- macOS: `target/release/librustbridge_jni.dylib`
- Windows: `target/release/rustbridge_jni.dll`

## Loading a Plugin

### From Bundle (Recommended)

```java
import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginConfig;
import com.rustbridge.jni.JniPluginLoader;
import java.nio.file.Path;

public class Example {
    public static void main(String[] args) throws Exception {
        // Load bundle and extract library for current platform
        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath("my-plugin-1.0.0.rbp")
            .verifySignatures(false)  // Set true for production
            .build();

        Path libraryPath = bundleLoader.extractLibrary();

        // Load the plugin
        try (Plugin plugin = JniPluginLoader.load(libraryPath.toString())) {
            String response = plugin.call("echo", "{\"message\": \"Hello\"}");
            System.out.println(response);
        }

        bundleLoader.close();
    }
}
```

### From Raw Library

```java
import com.rustbridge.Plugin;
import com.rustbridge.jni.JniPluginLoader;

// Platform-specific path
String pluginPath = "target/release/libmyplugin.so";  // Linux

try (Plugin plugin = JniPluginLoader.load(pluginPath)) {
    String response = plugin.call("echo", "{\"message\": \"Hello\"}");
    System.out.println(response);
}
```

## Setting Library Path

JNI requires the native library to be in `java.library.path`:

```bash
# Linux/macOS
java -Djava.library.path=target/release -jar myapp.jar

# Windows
java -Djava.library.path=target\release -jar myapp.jar
```

Or set `LD_LIBRARY_PATH` (Linux) / `DYLD_LIBRARY_PATH` (macOS):

```bash
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:target/release
java -jar myapp.jar
```

## Making JSON Calls

```java
try (Plugin plugin = JniPluginLoader.load(pluginPath)) {
    // Simple call
    String response = plugin.call("echo", "{\"message\": \"Hello, World!\"}");
    System.out.println(response);

    // With Gson for type-safe serialization
    Gson gson = new Gson();

    EchoRequest request = new EchoRequest("Hello");
    String requestJson = gson.toJson(request);

    String responseJson = plugin.call("echo", requestJson);
    EchoResponse response = gson.fromJson(responseJson, EchoResponse.class);

    System.out.println("Length: " + response.length);
}

class EchoRequest {
    String message;
    EchoRequest(String message) { this.message = message; }
}

class EchoResponse {
    String message;
    int length;
}
```

## Configuration

```java
import com.rustbridge.PluginConfig;
import com.rustbridge.LogLevel;

PluginConfig config = PluginConfig.defaults()
    .logLevel(LogLevel.DEBUG)
    .workerThreads(4)
    .maxConcurrentOps(100)
    .shutdownTimeoutMs(5000);

try (Plugin plugin = JniPluginLoader.load(pluginPath, config)) {
    // Plugin configured...
}
```

## Logging

```java
import com.rustbridge.LogCallback;

LogCallback callback = new LogCallback() {
    @Override
    public void log(String level, String target, String message) {
        System.out.printf("[%s] %s: %s%n", level, target, message);
    }
};

try (Plugin plugin = JniPluginLoader.load(pluginPath, config, callback)) {
    plugin.call("echo", "{\"message\": \"test\"}");
}
```

## Binary Transport (Advanced)

JNI supports binary transport for performance-critical paths:

```java
public static final int MSG_ECHO = 1;

// Create request as byte array
ByteBuffer request = ByteBuffer.allocate(268);
request.order(ByteOrder.LITTLE_ENDIAN);
request.put((byte) 1);  // version
request.put(new byte[3]);  // reserved
byte[] msgBytes = "Hello".getBytes(StandardCharsets.UTF_8);
request.put(msgBytes);
request.position(260);
request.putInt(msgBytes.length);

// Call binary transport
byte[] response = plugin.callRaw(MSG_ECHO, request.array());

// Parse response
ByteBuffer respBuf = ByteBuffer.wrap(response);
respBuf.order(ByteOrder.LITTLE_ENDIAN);
respBuf.position(264);
int length = respBuf.getInt();
System.out.println("Length: " + length);
```

## Error Handling

```java
import com.rustbridge.PluginException;

try {
    String response = plugin.call("invalid.type", "{}");
} catch (PluginException e) {
    System.err.println("Error code: " + e.getErrorCode());
    System.err.println("Message: " + e.getMessage());

    switch (e.getErrorCode()) {
        case 6:
            System.err.println("Unknown message type");
            break;
        case 7:
            System.err.println("Handler error");
            break;
        case 13:
            System.err.println("Too many concurrent requests");
            break;
        default:
            System.err.println("Unexpected error");
    }
}
```

## Java 8 Compatibility Notes

- Use `new LogCallback() { ... }` instead of lambdas if targeting Java 7
- Text blocks (`"""..."""`) require Java 15+; use string concatenation for older versions
- Records require Java 16+; use classes with getters/setters for older versions

```java
// Java 8 compatible
String request = "{\"message\": \"Hello\"}";
String response = plugin.call("echo", request);
```

## Performance: JNI vs FFM

| Metric | JNI | FFM |
|--------|-----|-----|
| Binary latency | ~398 ns | ~511 ns |
| JSON latency | ~4.1 μs | ~2.4 μs |
| Binary throughput | 2.6M ops/s | 1.8M ops/s |
| JSON throughput | 258K ops/s | 406K ops/s |

**Summary:**
- JNI is faster for binary transport
- FFM is faster for JSON transport
- Choose based on your workload

## Complete Example

```java
import com.rustbridge.*;
import com.rustbridge.jni.JniPluginLoader;
import com.google.gson.Gson;

public class CalculatorExample {
    static class AddRequest {
        long a;
        long b;
        AddRequest(long a, long b) { this.a = a; this.b = b; }
    }

    static class AddResponse {
        long result;
    }

    public static void main(String[] args) throws Exception {
        Gson gson = new Gson();

        PluginConfig config = PluginConfig.defaults()
            .logLevel(LogLevel.INFO);

        try (Plugin plugin = JniPluginLoader.load(
                "target/release/libcalculator_plugin.so",
                config)) {

            AddRequest request = new AddRequest(42, 58);
            String requestJson = gson.toJson(request);

            String responseJson = plugin.call("math.add", requestJson);
            AddResponse response = gson.fromJson(responseJson, AddResponse.class);

            System.out.println("42 + 58 = " + response.result);
        }
    }
}
```

## Troubleshooting

### UnsatisfiedLinkError

```
java.lang.UnsatisfiedLinkError: no rustbridge_jni in java.library.path
```

**Solution:** Ensure `java.library.path` includes the directory containing the JNI library:

```bash
java -Djava.library.path=target/release -jar myapp.jar
```

### Library Not Found

```
Plugin library not found: /path/to/plugin.so
```

**Solution:** Check the plugin path and ensure the file exists:

```java
File pluginFile = new File(pluginPath);
System.out.println("Exists: " + pluginFile.exists());
System.out.println("Absolute: " + pluginFile.getAbsolutePath());
```

## Related Documentation

- [JAVA_FFM.md](./JAVA_FFM.md) - Java 21+ FFM guide (recommended)
- [KOTLIN.md](./KOTLIN.md) - Kotlin-specific guide
- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership
