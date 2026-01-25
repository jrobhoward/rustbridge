# Java vs Kotlin Comparison

This document shows how Kotlin makes the rustbridge API more idiomatic and concise while using the same underlying Java
library.

## Basic Usage

### Java (Java 21+)

```java
import com.rustbridge.ffm.FfmPluginLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginConfig;
import com.google.gson.Gson;

public class Example {
    public static void main(String[] args) {
        PluginConfig config = PluginConfig.builder()
            .logLevel("info")
            .workerThreads(4)
            .build();

        Plugin plugin = null;
        try {
            plugin = FfmPluginLoader.load("libhello_plugin.so", config);

            Gson gson = new Gson();
            EchoRequest request = new EchoRequest("Hello");
            String requestJson = gson.toJson(request);

            String responseJson = plugin.call("echo", requestJson);
            EchoResponse response = gson.fromJson(responseJson, EchoResponse.class);

            System.out.println(response.message);

        } finally {
            if (plugin != null) {
                try {
                    plugin.close();
                } catch (Exception e) {
                    e.printStackTrace();
                }
            }
        }
    }
}

// Need full class definitions
class EchoRequest {
    private String message;

    public EchoRequest(String message) {
        this.message = message;
    }

    public String getMessage() { return message; }
    public void setMessage(String message) { this.message = message; }
}

class EchoResponse {
    private String message;
    private int length;

    // Constructors, getters, setters...
}
```

### Kotlin

```kotlin
import com.rustbridge.ffm.FfmPluginLoader
import com.google.gson.Gson

// Concise data classes
data class EchoRequest(val message: String)
data class EchoResponse(val message: String, val length: Int)

// Extension function for type safety
inline fun <reified T> Plugin.callTyped(messageType: String, request: Any): T {
    val gson = Gson()
    val requestJson = gson.toJson(request)
    val responseJson = this.call(messageType, requestJson)
    return gson.fromJson(responseJson, T::class.java)
}

fun main() {
    val config = PluginConfig.builder()
        .logLevel("info")
        .workerThreads(4)
        .build()

    // Use block handles cleanup automatically
    FfmPluginLoader.load("libhello_plugin.so", config).use { plugin ->
        val response = plugin.callTyped<EchoResponse>(
            "echo",
            EchoRequest("Hello")
        )
        println(response.message)
    }
    // Automatic cleanup!
}
```

## Error Handling

### Java

```java
try {
    String response = plugin.call("echo", requestJson);
    EchoResponse result = gson.fromJson(response, EchoResponse.class);
    System.out.println("Success: " + result.message);
} catch (PluginException e) {
    System.err.println("Error: " + e.getMessage());
    System.err.println("Code: " + e.getErrorCode());
}
```

### Kotlin

```kotlin
// Option 1: Traditional try-catch
try {
    val response = plugin.callTyped<EchoResponse>("echo", request)
    println("Success: ${response.message}")
} catch (e: PluginException) {
    println("Error: ${e.message} (code: ${e.errorCode})")
}

// Option 2: runCatching
runCatching {
    plugin.callTyped<EchoResponse>("echo", request)
}.onSuccess { response ->
    println("Success: ${response.message}")
}.onFailure { e ->
    println("Error: ${e.message}")
}

// Option 3: Sealed classes for type-safe results
when (val result = plugin.callSafe<EchoResponse>("echo", request)) {
    is PluginResult.Success -> println("Success: ${result.value.message}")
    is PluginResult.Error -> println("Error: ${result.exception.message}")
}

// Option 4: Result type with fallback
val response = plugin.callResult<EchoResponse>("echo", request)
    .getOrElse { EchoResponse("Fallback", 0) }
```

## Logging Callbacks

### Java

```java
LogCallback callback = new LogCallback() {
    @Override
    public void log(LogLevel level, String target, String message) {
        System.out.println("[" + level + "] " + message);
    }
};

PluginConfig config = PluginConfig.builder()
    .logLevel("info")
    .logCallback(callback)
    .build();
```

### Kotlin

```kotlin
// Option 1: Object expression
val callback = object : LogCallback {
    override fun log(level: LogLevel, target: String, message: String) {
        println("[$level] $message")
    }
}

// Option 2: SAM conversion (if LogCallback is functional interface)
val config = PluginConfig.builder()
    .logLevel("info")
    .logCallback { level, target, message ->
        println("[$level] $message")
    }
    .build()

// Option 3: Named class with state
class CountingLogCallback : LogCallback {
    var count = 0
        private set

    override fun log(level: LogLevel, target: String, message: String) {
        count++
        println("[$count] [$level] $message")
    }
}
```

## Collection Operations

### Java

```java
List<String> messages = Arrays.asList("Hello", "World", "Java");
List<EchoResponse> responses = new ArrayList<>();

for (String msg : messages) {
    EchoRequest request = new EchoRequest(msg);
    String requestJson = gson.toJson(request);
    String responseJson = plugin.call("echo", requestJson);
    EchoResponse response = gson.fromJson(responseJson, EchoResponse.class);
    responses.add(response);
}

List<String> results = new ArrayList<>();
for (EchoResponse response : responses) {
    results.add(response.message);
}
```

### Kotlin

```kotlin
val messages = listOf("Hello", "World", "Kotlin")

val results = messages
    .map { msg -> plugin.callTyped<EchoResponse>("echo", EchoRequest(msg)) }
    .map { it.message }

// Or with error handling:
val results = messages
    .map { msg -> plugin.callResult<EchoResponse>("echo", EchoRequest(msg)) }
    .mapNotNull { it.getOrNull() }
    .map { it.message }
```

## Key Kotlin Advantages

| Feature                 | Java                    | Kotlin             | Benefit                                          |
|-------------------------|-------------------------|--------------------|--------------------------------------------------|
| **Data classes**        | ~20 lines               | 1 line             | Concise, auto-generates equals/hashCode/toString |
| **Resource management** | try-finally             | `use` block        | Automatic cleanup, no boilerplate                |
| **Null safety**         | `@Nullable` annotations | Built-in `?`       | Compile-time null checking                       |
| **Type inference**      | Explicit types          | `val`/`var`        | Less verbose                                     |
| **Extension functions** | Utility classes         | Extensions         | Cleaner API                                      |
| **String templates**    | String concatenation    | `"${var}"`         | Readable string formatting                       |
| **Collections**         | Verbose streams         | Built-in operators | Functional, concise                              |
| **When expressions**    | switch statements       | `when`             | Exhaustive, returns value                        |
| **Sealed classes**      | Enums/inheritance       | `sealed`           | Type-safe alternatives                           |
| **Coroutines**          | CompletableFuture       | `suspend fun`      | Natural async/await                              |

## Summary

Kotlin provides:

- **Less boilerplate**: Data classes, type inference, smart casts
- **Better safety**: Null safety, when expressions, sealed classes
- **More expressiveness**: Extension functions, operator overloading, DSLs
- **Cleaner code**: Use blocks, string templates, collection operators

All while using the **same Java library** underneath - no special Kotlin wrapper needed!
