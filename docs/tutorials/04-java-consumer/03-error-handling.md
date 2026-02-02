# Section 3: Error Handling

In this section, you'll handle plugin errors gracefully in Java.

## Plugin Errors

When a plugin returns an error (e.g., prettifying invalid JSON), the `plugin.call()` method throws a `PluginException`.

## Add Error Handling

Update the `call` helper to handle errors:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginException;
import com.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;

public class Main {
    record ValidateRequest(String json) {}
    record ValidateResponse(boolean valid) {}

    record PrettifyRequest(String json, int indent) {}
    record PrettifyResponse(String result) {}

    private static final Gson gson = new Gson();

    public static void main(String[] args) throws Exception {
        String bundlePath = "json-plugin-0.1.0.rbp";

        var bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        var libraryPath = bundleLoader.extractLibrary();

        try (var plugin = FfmPluginLoader.load(libraryPath.toString())) {
            // Successful prettify
            var validReq = new PrettifyRequest("{\"name\":\"test\"}", 2);
            var validResp = call(plugin, "prettify", validReq, PrettifyResponse.class);
            System.out.println("Success:\n" + validResp.result());

            // This will fail - invalid JSON
            System.out.println("\nAttempting to prettify invalid JSON...");
            var invalidReq = new PrettifyRequest("{broken}", 2);
            var invalidResp = call(plugin, "prettify", invalidReq, PrettifyResponse.class);
            System.out.println("Result: " + invalidResp.result());

        } catch (PluginException e) {
            System.err.println("Plugin error: " + e.getMessage());
            System.err.println("Error code: " + e.getErrorCode());
        }

        bundleLoader.close();
    }

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
Success:
{
  "name": "test"
}

Attempting to prettify invalid JSON...
Plugin error: Invalid JSON: expected value at line 1 column 2
Error code: 3
```

## Error Codes

The `PluginException` includes an error code that maps to rustbridge error types:

| Code | Error Type                         |
|------|------------------------------------|
| 1    | Initialization error               |
| 2    | Unknown message type               |
| 3    | Handler error (your custom errors) |
| 4    | Serialization error                |
| 5    | Shutdown error                     |

## Graceful Error Handling

For production code, handle errors per-call rather than letting them bubble up:

```java
private static <Req, Resp> Resp callSafe(
        Plugin plugin,
        String messageType,
        Req request,
        Class<Resp> responseClass) {
    try {
        String requestJson = gson.toJson(request);
        String responseJson = plugin.call(messageType, requestJson);
        return gson.fromJson(responseJson, responseClass);
    } catch (PluginException e) {
        System.err.println("Plugin call failed: " + e.getMessage());
        return null;
    }
}
```

## Complete Example

Here's the final `Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.PluginException;
import com.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;

public class Main {
    record ValidateRequest(String json) {}
    record ValidateResponse(boolean valid) {}

    record PrettifyRequest(String json, int indent) {}
    record PrettifyResponse(String result) {}

    private static final Gson gson = new Gson();

    public static void main(String[] args) throws Exception {
        String bundlePath = "json-plugin-0.1.0.rbp";

        var bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        var libraryPath = bundleLoader.extractLibrary();

        try (var plugin = FfmPluginLoader.load(libraryPath.toString())) {
            // Validate various JSON strings
            testValidate(plugin, "{\"valid\": true}", "Valid object");
            testValidate(plugin, "[1, 2, 3]", "Valid array");
            testValidate(plugin, "{broken}", "Invalid JSON");

            System.out.println();

            // Prettify valid JSON
            testPrettify(plugin, "{\"a\":1,\"b\":2}", 2, "Compact object");

            // Try to prettify invalid JSON (will fail)
            testPrettify(plugin, "{broken}", 2, "Invalid JSON");
        }

        bundleLoader.close();
    }

    private static void testValidate(Plugin plugin, String json, String label)
            throws PluginException {
        var request = new ValidateRequest(json);
        var response = call(plugin, "validate", request, ValidateResponse.class);
        System.out.println(label + ": " + (response.valid() ? "valid" : "invalid"));
    }

    private static void testPrettify(Plugin plugin, String json, int indent, String label) {
        try {
            var request = new PrettifyRequest(json, indent);
            var response = call(plugin, "prettify", request, PrettifyResponse.class);
            System.out.println(label + ":\n" + response.result());
        } catch (PluginException e) {
            System.out.println(label + ": ERROR - " + e.getMessage());
        }
    }

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

## Summary

You've built a Java application that:

- Loads a rustbridge plugin bundle
- Makes type-safe calls using records and Gson
- Handles plugin errors gracefully

## Next Steps

Continue to [Chapter 5: Production Bundles](../05-production-bundles/README.md) to learn about:

- Embedding JSON schemas in bundles
- Code signing with minisign
- Software Bill of Materials (SBOM)
- Build metadata
