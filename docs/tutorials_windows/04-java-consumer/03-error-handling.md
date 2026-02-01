# Section 3: Error Handling

In this section, you'll add proper error handling for plugin exceptions.

## Understanding Error Types

There are two types of errors:

1. **Validation errors** - Returned as structured responses (e.g., invalid JSON syntax)
2. **Plugin exceptions** - Thrown when something goes wrong (e.g., invalid request)

## Create a Wrapper Class

Create `src\main\java\com\example\JsonPluginClient.java`:

```java
package com.example;

import com.rustbridge.Plugin;
import com.rustbridge.PluginException;
import com.google.gson.Gson;
import com.google.gson.FieldNamingPolicy;
import com.google.gson.GsonBuilder;
import java.io.Closeable;

/**
 * Type-safe wrapper for the JSON plugin.
 */
public class JsonPluginClient implements Closeable {

    private static final Gson gson = new GsonBuilder()
        .setFieldNamingPolicy(FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
        .create();

    private final Plugin plugin;

    public JsonPluginClient(Plugin plugin) {
        this.plugin = plugin;
    }

    /**
     * Validate a JSON string.
     *
     * @param json The JSON string to validate
     * @return Validation result with error details if invalid
     */
    public ValidateResponse validate(String json) {
        ValidateRequest req = new ValidateRequest(json);
        String response = plugin.call("validate", gson.toJson(req));
        return gson.fromJson(response, ValidateResponse.class);
    }

    /**
     * Prettify a JSON string.
     *
     * @param json The JSON string to prettify
     * @return Prettified JSON
     * @throws JsonPluginException if the JSON is invalid
     */
    public String prettify(String json) throws JsonPluginException {
        return prettify(json, 2);
    }

    /**
     * Prettify a JSON string with custom indentation.
     *
     * @param json The JSON string to prettify
     * @param indent Number of spaces for indentation (0-8)
     * @return Prettified JSON
     * @throws JsonPluginException if the JSON is invalid or indent is out of range
     */
    public String prettify(String json, int indent) throws JsonPluginException {
        try {
            PrettifyRequest req = new PrettifyRequest(json, indent);
            String response = plugin.call("prettify", gson.toJson(req));
            PrettifyResponse result = gson.fromJson(response, PrettifyResponse.class);
            return result.json;
        } catch (PluginException e) {
            throw new JsonPluginException("Prettify failed: " + e.getMessage(), e);
        }
    }

    /**
     * Minify a JSON string.
     *
     * @param json The JSON string to minify
     * @return Minify result with bytes saved
     * @throws JsonPluginException if the JSON is invalid
     */
    public MinifyResult minify(String json) throws JsonPluginException {
        try {
            MinifyRequest req = new MinifyRequest(json);
            String response = plugin.call("minify", gson.toJson(req));
            MinifyResponse result = gson.fromJson(response, MinifyResponse.class);
            return new MinifyResult(result.json, result.bytesSaved);
        } catch (PluginException e) {
            throw new JsonPluginException("Minify failed: " + e.getMessage(), e);
        }
    }

    @Override
    public void close() {
        plugin.close();
    }

    // Result types

    public record MinifyResult(String json, int bytesSaved) {
        public double compressionRatio() {
            int originalSize = json.length() + bytesSaved;
            return 1.0 - (double) json.length() / originalSize;
        }
    }
}

/**
 * Exception thrown by JsonPluginClient operations.
 */
class JsonPluginException extends Exception {
    public JsonPluginException(String message, Throwable cause) {
        super(message, cause);
    }
}
```

## Update Main to Use the Wrapper

Replace `src\main\java\com\example\Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.ffm.FfmPluginLoader;

public class Main {

    public static void main(String[] args) throws Exception {
        System.out.println("=== Java Consumer - Error Handling Demo ===\n");

        String bundlePath = "json-plugin-0.1.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        var plugin = FfmPluginLoader.load(bundleLoader.extractLibrary().toString());

        try (var client = new JsonPluginClient(plugin)) {

            // Demo 1: Validation (returns structured result, not exception)
            System.out.println("Demo 1: Validation results");
            testValidation(client, "{\"valid\": true}");
            testValidation(client, "{broken}");

            // Demo 2: Prettify with valid JSON
            System.out.println("\nDemo 2: Prettify valid JSON");
            try {
                String pretty = client.prettify("{\"a\":1}", 4);
                System.out.println("  Success:");
                for (String line : pretty.split("\n")) {
                    System.out.println("    " + line);
                }
            } catch (JsonPluginException e) {
                System.out.println("  Error: " + e.getMessage());
            }

            // Demo 3: Prettify with invalid JSON (throws exception)
            System.out.println("\nDemo 3: Prettify invalid JSON");
            try {
                client.prettify("{invalid}");
                System.out.println("  Should not reach here");
            } catch (JsonPluginException e) {
                System.out.println("  Caught expected error: " + e.getMessage());
            }

            // Demo 4: Prettify with invalid indent (throws exception)
            System.out.println("\nDemo 4: Invalid indent value");
            try {
                client.prettify("{\"a\":1}", 20);  // Max is 8
                System.out.println("  Should not reach here");
            } catch (JsonPluginException e) {
                System.out.println("  Caught expected error: " + e.getMessage());
            }

            // Demo 5: Minify with compression stats
            System.out.println("\nDemo 5: Minify with stats");
            String pretty = """
                {
                    "name": "Alice",
                    "scores": [100, 95, 88]
                }
                """;
            try {
                var result = client.minify(pretty);
                System.out.printf("  Original: %d bytes%n", pretty.length());
                System.out.printf("  Minified: %d bytes%n", result.json().length());
                System.out.printf("  Saved: %d bytes (%.1f%% reduction)%n",
                    result.bytesSaved(),
                    result.compressionRatio() * 100);
            } catch (JsonPluginException e) {
                System.out.println("  Error: " + e.getMessage());
            }
        }

        bundleLoader.close();
        System.out.println("\n=== Demo Complete ===");
    }

    private static void testValidation(JsonPluginClient client, String json) {
        ValidateResponse result = client.validate(json);
        String preview = json.length() > 30 ? json.substring(0, 27) + "..." : json;

        if (result.valid) {
            System.out.println("  ✓ " + preview);
        } else {
            System.out.printf("  ✗ %s - %s (line %d, col %d)%n",
                preview, result.error, result.line, result.column);
        }
    }
}
```

## Build and Run

```powershell
.\gradlew.bat run
```

Expected output:

```
=== Java Consumer - Error Handling Demo ===

Demo 1: Validation results
  ✓ {"valid": true}
  ✗ {broken} - expected ':' at line 1 column 8 (line 1, col 8)

Demo 2: Prettify valid JSON
  Success:
    {
        "a": 1
    }

Demo 3: Prettify invalid JSON
  Caught expected error: Prettify failed: Invalid JSON: ...

Demo 4: Invalid indent value
  Caught expected error: Prettify failed: Indent must be 0-8 spaces

Demo 5: Minify with stats
  Original: 73 bytes
  Minified: 41 bytes
  Saved: 32 bytes (43.8% reduction)

=== Demo Complete ===
```

## Summary

You've learned:

1. **Validation vs Exceptions** - Use validation responses for user input, exceptions for programming errors
2. **Wrapper Classes** - Provide type-safe, idiomatic APIs
3. **Error Handling** - Catch and wrap plugin exceptions appropriately

## What's Next?

Continue to [Chapter 5: Production Bundles](../05-production-bundles/README.md) to prepare your plugins for production.
