# Section 1: Project Setup

In this section, you'll create a Kotlin project with the rustbridge dependencies.

## Create the Consumer Directory

```powershell
cd $env:USERPROFILE\rustbridge-workspace\regex-plugin
mkdir consumers\kotlin
cd consumers\kotlin
```

## Initialize Gradle Project

Create `build.gradle.kts`:

```kotlin
plugins {
    kotlin("jvm") version "2.0.21"
    kotlin("plugin.serialization") version "2.0.21"
    application
}

group = "com.example"
version = "0.1.0"

repositories {
    mavenCentral()
    mavenLocal() // For local rustbridge development
}

dependencies {
    // Rustbridge FFM (requires Java 21+)
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-ffm:0.7.0")
    implementation("com.rustbridge:rustbridge-kotlin:0.7.0")

    // JSON serialization
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")

    // Testing
    testImplementation(kotlin("test"))
}

application {
    mainClass.set("com.example.MainKt")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

tasks.test {
    useJUnitPlatform()
}
```

Create `settings.gradle.kts`:

```kotlin
rootProject.name = "regex-plugin-kotlin"
```

## Create the Package Structure

```powershell
mkdir -p src\main\kotlin\com\example
mkdir -p src\test\kotlin\com\example
```

## Create the Main Class

Create `src\main\kotlin\com\example\Main.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader
import kotlinx.serialization.json.Json

fun main() {
    println("Kotlin Consumer - Regex Plugin Demo")

    val bundlePath = "..\\..\\regex-plugin-0.1.0.rbp"

    // Load the bundle
    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary().toString()
    println("Extracted library: $libraryPath")

    // Load the plugin
    FfmPluginLoader.load(libraryPath).use { plugin ->
        println("Plugin loaded successfully!")

        // Simple echo test
        val request = """{"pattern": "\\d+", "text": "abc123def456"}"""
        val response = plugin.call("match", request)

        println("Response: $response")
    }

    bundleLoader.close()
    println("Done!")
}
```

## Copy the Bundle

Copy the regex plugin bundle to the consumers directory:

```powershell
cd $env:USERPROFILE\rustbridge-workspace\regex-plugin
Copy-Item regex-plugin-0.1.0.rbp consumers\kotlin\
```

## Install rustbridge Java Libraries

If you haven't already, install the rustbridge Java libraries to Maven local:

```powershell
cd $env:USERPROFILE\rustbridge-workspace\rustbridge\rustbridge-java
.\gradlew.bat publishToMavenLocal
```

## Build and Run

```powershell
cd $env:USERPROFILE\rustbridge-workspace\regex-plugin\consumers\kotlin
.\gradlew.bat run
```

Expected output:

```
Kotlin Consumer - Regex Plugin Demo
Extracted library: C:\Users\...\regex_plugin.dll
Plugin loaded successfully!
Response: {"matched":true,"match_text":"123","start":3,"end":6}
Done!
```

## Understanding the Setup

### Java 21+ Requirement

The FFM API is stable in Java 21+:

```kotlin
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}
```

### Bundle Loading

`BundleLoader` extracts the correct native library for your platform:

```kotlin
val bundleLoader = BundleLoader.builder()
    .bundlePath(bundlePath)
    .verifySignatures(false)  // Skip signature check for development
    .build()
```

### Plugin Lifecycle

The plugin is loaded and automatically closed with Kotlin's `use`:

```kotlin
FfmPluginLoader.load(libraryPath).use { plugin ->
    // Plugin is available here
}  // Automatically closed
```

## What's Next?

In the next section, you'll make structured calls with proper request/response types.

[Continue to Section 2: Calling the Plugin →](./02-calling-plugin.md)
