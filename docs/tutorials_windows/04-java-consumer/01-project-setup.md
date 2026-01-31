# Section 1: Project Setup

In this section, you'll create a Java project to consume the JSON plugin.

## Create the Consumer Directory

```powershell
cd $env:USERPROFILE\rustbridge-workspace\json-plugin
mkdir consumers\java-ffm
cd consumers\java-ffm
```

## Initialize Gradle Project

Create `build.gradle.kts`:

```kotlin
plugins {
    java
    application
}

group = "com.example"
version = "1.0.0"

repositories {
    mavenCentral()
    mavenLocal()
}

dependencies {
    // Rustbridge (choose one based on your Java version)
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-ffm:0.7.0")  // Java 21+
    // OR: implementation("com.rustbridge:rustbridge-jni:0.7.0")  // Java 17+

    // JSON
    implementation("com.google.code.gson:gson:2.11.0")
}

application {
    mainClass.set("com.example.Main")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))  // or 17 for JNI
    }
}
```

Create `settings.gradle.kts`:

```kotlin
rootProject.name = "json-plugin-java"
```

## Create Package Structure

```powershell
mkdir -p src\main\java\com\example
```

## Create Main Class

Create `src\main\java\com\example\Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.Plugin;
import com.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

public class Main {

    private static final Gson gson = new GsonBuilder()
        .setPrettyPrinting()
        .create();

    public static void main(String[] args) throws Exception {
        System.out.println("Java Consumer - JSON Plugin Demo");
        System.out.println("================================\n");

        String bundlePath = "..\\..\\json-plugin-1.0.0.rbp";

        // Load the bundle
        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        String libraryPath = bundleLoader.extractLibrary().toString();
        System.out.println("Extracted library: " + libraryPath);

        // Load the plugin
        try (Plugin plugin = FfmPluginLoader.load(libraryPath)) {
            System.out.println("Plugin loaded successfully!\n");

            // Test validation
            testValidation(plugin);
        }

        bundleLoader.close();
        System.out.println("\nDone!");
    }

    private static void testValidation(Plugin plugin) {
        System.out.println("Testing JSON validation:");

        // Valid JSON
        String validJson = "{\"name\": \"test\", \"value\": 42}";
        String request = gson.toJson(new ValidateRequest(validJson));
        String response = plugin.call("validate", request);
        ValidateResponse result = gson.fromJson(response, ValidateResponse.class);

        System.out.println("  Valid JSON: " + result.valid);

        // Invalid JSON
        String invalidJson = "{\"name\": }";
        request = gson.toJson(new ValidateRequest(invalidJson));
        response = plugin.call("validate", request);
        result = gson.fromJson(response, ValidateResponse.class);

        System.out.println("  Invalid JSON: " + result.valid);
        System.out.println("  Error: " + result.error);
        System.out.println("  Position: line " + result.line + ", column " + result.column);
    }
}

// Request/Response classes
class ValidateRequest {
    String json;
    ValidateRequest(String json) { this.json = json; }
}

class ValidateResponse {
    boolean valid;
    String error;
    Integer line;
    Integer column;
}
```

## Copy the Bundle

```powershell
cd $env:USERPROFILE\rustbridge-workspace\json-plugin
Copy-Item json-plugin-1.0.0.rbp consumers\java-ffm\
```

## Build and Run

```powershell
cd consumers\java-ffm
.\gradlew.bat run
```

Expected output:

```
Java Consumer - JSON Plugin Demo
================================

Extracted library: C:\Users\...\json_plugin.dll
Plugin loaded successfully!

Testing JSON validation:
  Valid JSON: true
  Invalid JSON: false
  Error: expected value at line 1 column 10
  Position: line 1, column 10

Done!
```

## What's Next?

In the next section, you'll add more features and improve the code.

[Continue to Section 2: Calling the Plugin →](./02-calling-plugin.md)
