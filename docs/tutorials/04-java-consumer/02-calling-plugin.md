# Section 2: Calling the Plugin

In this section, you'll load and call the JSON plugin using type-safe Java records.

## Update Main.java

Replace the contents of `src/main/java/com/example/Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginException;
import com.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;

public class Main {
    // Request/Response records
    record ValidateRequest(String json) {}
    record ValidateResponse(boolean valid) {}

    record PrettifyRequest(String json, int indent) {}
    record PrettifyResponse(String result) {}

    private static final Gson gson = new Gson();

    public static void main(String[] args) throws Exception {
        String bundlePath = "json-plugin-1.0.0.rbp";

        // Load the bundle and extract the library for this platform
        var bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)  // We'll enable this in Chapter 5
            .build();

        var libraryPath = bundleLoader.extractLibrary();
        System.out.println("Loaded library from: " + libraryPath);

        try (var plugin = FfmPluginLoader.load(libraryPath.toString())) {
            // Validate some JSON
            var validateReq = new ValidateRequest("{\"name\": \"test\"}");
            var validateResp = call(plugin, "validate", validateReq, ValidateResponse.class);
            System.out.println("Is valid: " + validateResp.valid());

            // Prettify some JSON
            var prettifyReq = new PrettifyRequest("{\"name\":\"test\",\"value\":42}", 4);
            var prettifyResp = call(plugin, "prettify", prettifyReq, PrettifyResponse.class);
            System.out.println("Prettified:\n" + prettifyResp.result());

            // Validate invalid JSON
            var invalidReq = new ValidateRequest("{broken json}");
            var invalidResp = call(plugin, "validate", invalidReq, ValidateResponse.class);
            System.out.println("Invalid JSON is valid: " + invalidResp.valid());
        }

        bundleLoader.close();
    }

    /**
     * Type-safe plugin call helper.
     */
    private static <Req, Resp> Resp call(
            Plugin plugin,
            String messageType,
            Req request,
            Class<Resp> responseClass) throws PluginException {
        String requestJson = gson.toJson(request);
        String responseJson = plugin.call(messageType, requestJson);
        return gson.fromJson(responseJson, responseClass);
    }
}
```

## Run the Application

```bash
./gradlew run
```

Expected output:

```
Loaded library from: /tmp/rustbridge-bundles/json-plugin/1.0.0/release/libjson_plugin.so
Is valid: true
Prettified:
{
    "name": "test",
    "value": 42
}
Invalid JSON is valid: false
```

## Understanding the Code

### BundleLoader

```java
var bundleLoader = BundleLoader.builder()
    .bundlePath(bundlePath)
    .verifySignatures(false)
    .build();
```

The `BundleLoader`:

- Opens the `.rbp` ZIP file
- Reads the manifest to find the right library for your platform
- Extracts it to a temp directory
- Optionally verifies minisign signatures (covered in Chapter 5)

### FfmPluginLoader

```java
try (var plugin = FfmPluginLoader.load(libraryPath.toString())) {
    // use plugin...
}
```

The `FfmPluginLoader`:

- Uses Java 21's Foreign Function & Memory API
- Loads the native library
- Provides a `call(messageType, jsonPayload)` method
- Implements `AutoCloseable` for proper cleanup

### Java Records

```java
record ValidateRequest(String json) {}
record ValidateResponse(boolean valid) {}
```

Records provide:

- Immutable data classes
- Automatic `equals()`, `hashCode()`, `toString()`
- Accessor methods matching field names (e.g., `valid()`)

### Type-Safe Helper Method

```java
private static <Req, Resp> Resp call(
        Plugin plugin,
        String messageType,
        Req request,
        Class<Resp> responseClass) throws PluginException {
    String requestJson = gson.toJson(request);
    String responseJson = plugin.call(messageType, requestJson);
    return gson.fromJson(responseJson, responseClass);
}
```

This helper:

- Serializes any request object to JSON
- Calls the plugin with the message type
- Deserializes the response to the expected type
- Provides compile-time type safety

## What's Next?

In the next section, you'll handle plugin errors gracefully.

[Continue to Section 3: Error Handling →](./03-error-handling.md)
