# Section 2: Calling the Plugin

In this section, you'll implement all JSON plugin operations.

## Add Message Classes

Create `src\main\java\com\example\Messages.java`:

```java
package com.example;

import com.google.gson.annotations.SerializedName;

// ============================================================================
// Validate
// ============================================================================

class ValidateRequest {
    String json;

    ValidateRequest(String json) {
        this.json = json;
    }
}

class ValidateResponse {
    boolean valid;
    String error;
    Integer line;
    Integer column;
}

// ============================================================================
// Prettify
// ============================================================================

class PrettifyRequest {
    String json;
    int indent;

    PrettifyRequest(String json) {
        this(json, 2);
    }

    PrettifyRequest(String json, int indent) {
        this.json = json;
        this.indent = indent;
    }
}

class PrettifyResponse {
    String json;
}

// ============================================================================
// Minify
// ============================================================================

class MinifyRequest {
    String json;

    MinifyRequest(String json) {
        this.json = json;
    }
}

class MinifyResponse {
    String json;
    @SerializedName("bytes_saved")
    int bytesSaved;
}
```

## Update Main Class

Replace `src\main\java\com\example\Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

public class Main {

    private static final Gson gson = new GsonBuilder()
        .setFieldNamingPolicy(com.google.gson.FieldNamingPolicy.LOWER_CASE_WITH_UNDERSCORES)
        .create();

    public static void main(String[] args) throws Exception {
        System.out.println("=== Java Consumer - JSON Plugin Demo ===\n");

        String bundlePath = "json-plugin-0.1.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        String libraryPath = bundleLoader.extractLibrary().toString();

        try (Plugin plugin = FfmPluginLoader.load(libraryPath)) {

            // Demo 1: Validate JSON
            System.out.println("Demo 1: Validate JSON");
            validateDemo(plugin);

            // Demo 2: Prettify JSON
            System.out.println("\nDemo 2: Prettify JSON");
            prettifyDemo(plugin);

            // Demo 3: Minify JSON
            System.out.println("\nDemo 3: Minify JSON");
            minifyDemo(plugin);

            // Demo 4: Custom indentation
            System.out.println("\nDemo 4: Custom indentation");
            customIndentDemo(plugin);
        }

        bundleLoader.close();
        System.out.println("\n=== Demo Complete ===");
    }

    private static void validateDemo(Plugin plugin) {
        String[] testCases = {
            "{\"name\": \"Alice\", \"age\": 30}",
            "{\"invalid: json}",
            "[1, 2, 3, ]"
        };

        for (String json : testCases) {
            ValidateRequest req = new ValidateRequest(json);
            String response = plugin.call("validate", gson.toJson(req));
            ValidateResponse result = gson.fromJson(response, ValidateResponse.class);

            String preview = json.length() > 30 ? json.substring(0, 30) + "..." : json;
            if (result.valid) {
                System.out.println("  ✓ Valid: " + preview);
            } else {
                System.out.printf("  ✗ Invalid: %s%n", preview);
                System.out.printf("    Error at line %d, column %d: %s%n",
                    result.line, result.column, result.error);
            }
        }
    }

    private static void prettifyDemo(Plugin plugin) {
        String compact = "{\"users\":[{\"name\":\"Alice\",\"age\":30},{\"name\":\"Bob\",\"age\":25}]}";

        PrettifyRequest req = new PrettifyRequest(compact);
        String response = plugin.call("prettify", gson.toJson(req));
        PrettifyResponse result = gson.fromJson(response, PrettifyResponse.class);

        System.out.println("  Input:  " + compact);
        System.out.println("  Output:");
        for (String line : result.json.split("\n")) {
            System.out.println("    " + line);
        }
    }

    private static void minifyDemo(Plugin plugin) {
        String pretty = """
            {
              "name": "Alice",
              "email": "alice@example.com",
              "active": true
            }
            """;

        MinifyRequest req = new MinifyRequest(pretty);
        String response = plugin.call("minify", gson.toJson(req));
        MinifyResponse result = gson.fromJson(response, MinifyResponse.class);

        System.out.println("  Input size:  " + pretty.length() + " bytes");
        System.out.println("  Output size: " + result.json.length() + " bytes");
        System.out.println("  Saved: " + result.bytesSaved + " bytes");
        System.out.println("  Result: " + result.json);
    }

    private static void customIndentDemo(Plugin plugin) {
        String json = "{\"a\":1,\"b\":2}";

        int[] indents = {0, 2, 4};
        for (int indent : indents) {
            PrettifyRequest req = new PrettifyRequest(json, indent);
            String response = plugin.call("prettify", gson.toJson(req));
            PrettifyResponse result = gson.fromJson(response, PrettifyResponse.class);

            System.out.println("  Indent " + indent + ":");
            for (String line : result.json.split("\n")) {
                System.out.println("    |" + line);
            }
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
=== Java Consumer - JSON Plugin Demo ===

Demo 1: Validate JSON
  ✓ Valid: {"name": "Alice", "age": 30}
  ✗ Invalid: {"invalid: json}
    Error at line 1, column 11: expected ':' at line 1 column 11
  ✗ Invalid: [1, 2, 3, ]
    Error at line 1, column 10: trailing comma at line 1 column 10

Demo 2: Prettify JSON
  Input:  {"users":[{"name":"Alice","age":30},{"name":"Bob","age":25}]}
  Output:
    {
      "users": [
        {
          "name": "Alice",
          "age": 30
        },
        ...

Demo 3: Minify JSON
  Input size:  89 bytes
  Output size: 55 bytes
  Saved: 34 bytes
  Result: {"name":"Alice","email":"alice@example.com","active":true}

Demo 4: Custom indentation
  Indent 0:
    |{
    |"a": 1,
    |"b": 2
    |}
  ...

=== Demo Complete ===
```

## What's Next?

In the next section, you'll add proper error handling.

[Continue to Section 3: Error Handling →](./03-error-handling.md)
