# Section 1: Project Setup

In this section, you'll set up a Kotlin project to consume your regex plugin.

## Generate the Kotlin Consumer

```bash
cd $RUSTBRIDGE_WORKSPACE

rustbridge new regex-kotlin-app --kotlin
cd regex-kotlin-app/consumers/kotlin
```

## Verify the Project Structure

The `rustbridge new` command creates a Rust plugin at the root with consumers in the `consumers/` directory:

```
regex-kotlin-app/
├── Cargo.toml                 # Rust plugin
├── src/
│   └── lib.rs
└── consumers/
    └── kotlin/
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

> **Tip**: If you're a git user, at this point, you may want to run `git init`, `git add .` and `git commit` from
> `regex-kotlin-app/` (the root). At the end of each tutorial section, you can commit your progress.


> **Tip**: Now would also be a good time to load the project in your IDE or editor of choice.
> I recommend IntelliJ IDEA.

## Copy Your Plugin Bundle

Copy the bundle you created in Chapter 1:

```bash
# From consumers/kotlin/
cp $RUSTBRIDGE_WORKSPACE/regex-plugin/target/bundle/regex-plugin-0.1.0.rbp .
```

## Examine build.gradle.kts

The template's `build.gradle.kts` includes:

```kotlin
plugins {
    kotlin("jvm") version "2.3.0"
    application
}

repositories {
    mavenLocal()  // For rustbridge libraries
    mavenCentral()
}

dependencies {
    implementation("io.github.jrobhoward.rustbridge:rustbridge-core:0.10.0")
    implementation("io.github.jrobhoward.rustbridge:rustbridge-ffm:0.10.0")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.18.2")
}

kotlin {
    jvmToolchain(21)  // FFM requires Java 21+ (LTS: 21, 25)
}

application {
    mainClass.set("com.example.MainKt")
}

tasks.withType<JavaExec> {
    // Required for FFM native access
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
```

Key points:

- **mavenLocal()**: Finds rustbridge libraries you installed
- **rustbridge-ffm**: Uses FFM (Foreign Function & Memory API) for native access (Java 21+)
- **jackson-module-kotlin**: For JSON serialization
- **--enable-native-access**: Required JVM flag for FFM

## Build the Project

Verify everything compiles:

```bash
./gradlew build
```

You might see warnings about the echo plugin not being found - that's expected since we haven't updated Main.kt yet.

## Verify Your Bundle

You can inspect the bundle contents:

```bash
rustbridge bundle list regex-plugin-0.1.0.rbp
```

Output:

```
Bundle: regex-plugin v0.1.0
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
