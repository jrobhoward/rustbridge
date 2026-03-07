# Section 1: Project Setup

In this section, you'll set up the Java FFM consumer project that was generated with your plugin.

## Use the Generated Consumer

If you created your plugin with `--java-ffm` in Chapter 3, you already have a consumer project:

```bash
cd $RUSTBRIDGE_WORKSPACE/json-plugin/consumers/java-ffm
```

If you didn't, generate one now:

```bash
cd $RUSTBRIDGE_WORKSPACE
rustbridge new json-plugin --java-ffm
cd json-plugin/consumers/java-ffm
```

## Verify the Project Structure

```
consumers/java-ffm/
├── build.gradle.kts
├── settings.gradle.kts
├── gradle/
│   └── wrapper/
├── gradlew
├── gradlew.bat
└── src/
    └── main/
        └── java/
            └── com/
                └── example/
                    └── Main.java
```

## Copy Your Plugin Bundle

Copy the bundle you created in Chapter 3:

```bash
cp ../../target/bundle/json-plugin-0.1.0.rbp .
```

> **Note**: The bundle path is relative to the consumers/java-ffm directory.

## Install rustbridge Java Libraries

If you haven't already, install the Java libraries to Maven local:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
cd $RUSTBRIDGE_WORKSPACE/json-plugin/consumers/java-ffm
```

## Examine build.gradle.kts

The generated `build.gradle.kts` includes:

```kotlin
plugins {
    java
    application
}

group = "com.example"
version = "0.1.0"

application {
    mainClass.set("com.example.Main")
}

repositories {
    mavenLocal()  // For local rustbridge development
    mavenCentral()
}

dependencies {
    // rustbridge dependencies
    implementation("io.github.jrobhoward.rustbridge:rustbridge-core:0.10.0")
    implementation("io.github.jrobhoward.rustbridge:rustbridge-ffm:0.10.0")

    // JSON serialization
    implementation("com.google.code.gson:gson:2.10.1")

    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
}

java {
    toolchain {
        // FFM requires Java 21+
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

// Check if running Java 21 (needs --enable-preview for FFM)
val needsPreview = provider {
    java.toolchain.languageVersion.get().asInt() == 21
}

tasks.withType<JavaCompile> {
    if (needsPreview.get()) {
        options.compilerArgs.add("--enable-preview")
    }
}

tasks.withType<JavaExec> {
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
```

Key points:

- **mavenLocal()**: Finds rustbridge libraries you installed
- **rustbridge-ffm**: Uses Java's Foreign Function & Memory API
- **gson**: For JSON serialization (simpler than Jackson for Java)
- **needsPreview**: Automatically detects Java 21 and adds `--enable-preview`
- **jvmArgs**: Required for FFM native access

## Build the Project

Verify everything compiles:

```bash
./gradlew build
```

## What's Next?

In the next section, you'll update Main.java to load and call the JSON plugin.

[Continue to Section 2: Calling the Plugin →](./02-calling-plugin.md)
