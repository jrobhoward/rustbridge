# Section 1: Project Setup

In this section, you'll set up a Kotlin project to consume your regex plugin.

## Copy the Kotlin Template

```bash
cd ~/rustbridge-workspace

cp -r rustbridge/templates/kotlin regex-kotlin-app
cd regex-kotlin-app
```

## Verify the Project Structure

```
regex-kotlin-app/
├── build.gradle.kts
├── settings.gradle.kts
├── gradle/
├── gradlew
├── gradlew.bat
└── src/
    └── main/
        └── kotlin/
            └── com/
                └── example/
                    └── Main.kt
```

## Copy Your Plugin Bundle

Copy the bundle you created in Chapter 1:

```bash
cp ~/rustbridge-workspace/regex-plugin/regex-plugin-1.0.0.rbp .
```

## Install rustbridge Java Libraries

If you haven't already, install the Java libraries to Maven local:

```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
cd ~/rustbridge-workspace/regex-kotlin-app
```

## Examine build.gradle.kts

The template's `build.gradle.kts` includes:

```kotlin
plugins {
    kotlin("jvm") version "2.0.0"
    application
}

repositories {
    mavenLocal()  // For rustbridge libraries
    mavenCentral()
}

dependencies {
    implementation("com.rustbridge:rustbridge-core:0.6.0")
    implementation("com.rustbridge:rustbridge-ffm:0.6.0")
    implementation("com.rustbridge:rustbridge-kotlin:0.6.0")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.17.2")
}

application {
    mainClass.set("com.example.MainKt")
}

tasks.withType<JavaExec> {
    // Required for Foreign Function & Memory API
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
```

Key points:

- **mavenLocal()**: Finds rustbridge libraries you installed
- **rustbridge-ffm**: Uses Java 21's Foreign Function API (faster than JNI)
- **jackson-module-kotlin**: For JSON serialization
- **jvmArgs**: Required for FFM access

## Build the Project

Verify everything compiles:

```bash
./gradlew build
```

You might see warnings about the echo plugin not being found - that's expected since we haven't updated Main.kt yet.

## Verify Your Bundle

You can inspect the bundle contents:

```bash
cd ~/rustbridge-workspace/rustbridge
cargo run -p rustbridge-cli -- bundle list ../regex-kotlin-app/regex-plugin-1.0.0.rbp
```

Output:

```
Bundle: regex-plugin-1.0.0.rbp
  Name: regex-plugin
  Version: 1.0.0
  Libraries:
    - linux-x86_64: libs/linux-x86_64/libregex_plugin.so
```

## What's Next?

In the next section, you'll update Main.kt to load and call the regex plugin.

[Continue to Section 2: Calling the Plugin →](./02-calling-plugin.md)
