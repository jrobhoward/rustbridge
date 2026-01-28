# Getting Started: Java 21+ (FFM)

This guide walks you through using rustbridge plugins from Java 21+ using the Foreign Function & Memory API (FFM).

## Prerequisites

- **Java 21 or later** - FFM requires Java 21+
  ```bash
  java --version  # Should be >= 21
  ```
- **Gradle or Maven** - For dependency management
- **A rustbridge plugin** - Either a `.rbp` bundle or `.so`/`.dylib`/`.dll` file

## Add Dependencies

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("com.rustbridge:rustbridge-core:0.5.0")
    implementation("com.rustbridge:rustbridge-ffm:0.5.0")
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'com.rustbridge:rustbridge-core:0.5.0'
    implementation 'com.rustbridge:rustbridge-ffm:0.5.0'
}
```

### Maven

```xml
<dependencies>
    <dependency>
        <groupId>com.rustbridge</groupId>
        <artifactId>rustbridge-core</artifactId>
        <version>0.5.0</version>
    </dependency>
    <dependency>
        <groupId>com.rustbridge</groupId>
        <artifactId>rustbridge-ffm</artifactId>
        <version>0.5.0</version>
    </dependency>
</dependencies>
```

## Local Development

When working with rustbridge source code (not published to Maven Central), publish to MavenLocal first:

```bash
cd rustbridge-java
./gradlew publishToMavenLocal
```

Then add the `mavenLocal()` repository to your build:

**Gradle (Kotlin DSL)**

```kotlin
repositories {
    mavenLocal()
    mavenCentral()
}
```

**Maven**

```xml
<repositories>
    <repository>
        <id>local</id>
        <url>file://${user.home}/.m2/repository</url>
    </repository>
</repositories>
```

## Loading a Plugin

### From Bundle (Recommended)

```java
import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginConfig;
import com.rustbridge.ffm.FfmPluginLoader;
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
        try (Plugin plugin = FfmPluginLoader.load(libraryPath.toString())) {
            // Use the plugin...
            String response = plugin.call("echo", """{"message": "Hello"}""");
            System.out.println(response);
        }

        bundleLoader.close();
    }
}
```

### From Raw Library

```java
import com.rustbridge.Plugin;
import com.rustbridge.ffm.FfmPluginLoader;

// Platform-specific path
String pluginPath = "target/release/libmyplugin.so";  // Linux
// String pluginPath = "target/release/libmyplugin.dylib";  // macOS
// String pluginPath = "target/release/myplugin.dll";  // Windows

try (Plugin plugin = FfmPluginLoader.load(pluginPath)) {
    String response = plugin.call("echo", """{"message": "Hello"}""");
    System.out.println(response);
}
```

## Making JSON Calls

```java
try (Plugin plugin = FfmPluginLoader.load(pluginPath)) {
    // Simple call
    String response = plugin.call("echo", """{"message": "Hello, World!"}""");
    System.out.println(response);
    // Output: {"message":"Hello, World!","length":13}

    // With Jackson for type-safe serialization
    ObjectMapper mapper = new ObjectMapper();

    record EchoRequest(String message) {}
    record EchoResponse(String message, int length) {}

    EchoRequest request = new EchoRequest("Hello");
    String requestJson = mapper.writeValueAsString(request);

    String responseJson = plugin.call("echo", requestJson);
    EchoResponse response = mapper.readValue(responseJson, EchoResponse.class);

    System.out.println("Length: " + response.length());
}
```

## Configuration

```java
import com.rustbridge.PluginConfig;
import com.rustbridge.LogLevel;

PluginConfig config = PluginConfig.defaults()
    .logLevel(LogLevel.DEBUG)           // Log level
    .workerThreads(4)                   // Async worker threads
    .maxConcurrentOps(100)              // Concurrency limit
    .shutdownTimeoutMs(5000);           // Shutdown timeout

try (Plugin plugin = FfmPluginLoader.load(pluginPath, config)) {
    // Plugin configured...
}
```

## Logging

Receive logs from the Rust plugin:

```java
import com.rustbridge.LogCallback;

LogCallback callback = (level, target, message) -> {
    System.out.printf("[%s] %s: %s%n", level, target, message);
};

try (Plugin plugin = FfmPluginLoader.load(pluginPath, config, callback)) {
    // Logs from Rust will be forwarded to your callback
    plugin.call("echo", """{"message": "test"}""");
}

// Change log level dynamically
plugin.setLogLevel(LogLevel.DEBUG);
```

## Binary Transport (Advanced)

For performance-critical paths, use binary transport:

```java
import java.lang.foreign.*;

public static final int MSG_ECHO = 1;

public static final StructLayout ECHO_REQUEST_LAYOUT = MemoryLayout.structLayout(
    ValueLayout.JAVA_BYTE.withName("version"),
    MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
    MemoryLayout.sequenceLayout(256, ValueLayout.JAVA_BYTE).withName("message"),
    ValueLayout.JAVA_INT.withName("message_len")
);

try (Arena arena = Arena.ofConfined()) {
    // Allocate request
    MemorySegment request = arena.allocate(ECHO_REQUEST_LAYOUT);
    request.set(ValueLayout.JAVA_BYTE, 0, (byte) 1);  // version

    // Set message
    String msg = "Hello";
    byte[] msgBytes = msg.getBytes(StandardCharsets.UTF_8);
    MemorySegment.copy(msgBytes, 0, request, 4, msgBytes.length);
    request.set(ValueLayout.JAVA_INT, 260, msgBytes.length);

    // Call binary transport
    MemorySegment response = plugin.callRaw(MSG_ECHO, request, 268);

    // Read response
    int length = response.get(ValueLayout.JAVA_INT, 264);
    System.out.println("Length: " + length);
}
```

### Zero-Copy Response

```java
// For maximum performance
RawResponse response = plugin.callRawZeroCopy(MSG_ECHO, request);
try {
    int length = response.segment().get(ValueLayout.JAVA_INT, 264);
} finally {
    response.close();  // Must free native memory
}
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
        case 6 -> System.err.println("Unknown message type");
        case 7 -> System.err.println("Handler error");
        case 13 -> System.err.println("Too many concurrent requests");
        default -> System.err.println("Unexpected error");
    }
}
```

## Monitoring

```java
// Check plugin state
LifecycleState state = plugin.getState();
System.out.println("State: " + state);  // ACTIVE

// Monitor rejected requests (due to concurrency limits)
long rejectedCount = plugin.getRejectedRequestCount();
if (rejectedCount > 0) {
    System.out.println("Rejected: " + rejectedCount + " requests");
}
```

## JVM Arguments

For FFM to work, you need to enable native access:

```bash
java --enable-preview --enable-native-access=ALL-UNNAMED -jar myapp.jar
```

Or in Gradle:

```kotlin
tasks.withType<JavaExec> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
```

## Complete Example

```java
import com.rustbridge.*;
import com.rustbridge.ffm.FfmPluginLoader;
import com.fasterxml.jackson.databind.ObjectMapper;

public class CalculatorExample {
    record AddRequest(long a, long b) {}
    record AddResponse(long result) {}

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = new ObjectMapper();

        PluginConfig config = PluginConfig.defaults()
            .logLevel(LogLevel.INFO);

        LogCallback logCallback = (level, target, message) -> {
            System.out.printf("[%s] %s%n", level, message);
        };

        try (Plugin plugin = FfmPluginLoader.load(
                "target/release/libcalculator_plugin.so",
                config,
                logCallback)) {

            // Make typed call
            AddRequest request = new AddRequest(42, 58);
            String requestJson = mapper.writeValueAsString(request);

            String responseJson = plugin.call("math.add", requestJson);
            AddResponse response = mapper.readValue(responseJson, AddResponse.class);

            System.out.println("42 + 58 = " + response.result());
        }
    }
}
```

## Related Documentation

- [JAVA_JNI.md](./JAVA_JNI.md) - Java 17+ JNI guide (legacy support)
- [KOTLIN.md](./KOTLIN.md) - Kotlin-specific guide
- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership
