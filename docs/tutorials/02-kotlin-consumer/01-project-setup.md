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

> **Tip**: If you're a git user, at this point, you may want to run `git init`, `git add .` and `git commit` at this
> time.
> At the end of each tutorial section, you can commit your progress.


> **Tip**: Now would also be a good time to load the project in your IDE or editor of choice.
> I recommend IntelliJ IDEA.

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
rustbridge bundle list regex-plugin-1.0.0.rbp
```

Output:

```
Bundle: regex-plugin v1.0.0
Bundle format: v1.0

Platforms:
  linux-x86_64:
    Variants: release

Files:
  manifest.json
  lib/linux-x86_64/release/libregex_plugin.so
```

## What's Next?

In the next section, you'll update Main.kt to load and call the regex plugin.

[Continue to Section 2: Calling the Plugin →](./02-calling-plugin.md)
