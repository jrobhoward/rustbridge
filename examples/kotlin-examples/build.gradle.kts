plugins {
    kotlin("jvm") version "2.0.21"
    application
}

group = "com.rustbridge.examples"
version = "0.7.0"

repositories {
    mavenCentral()
    mavenLocal() // For locally built rustbridge artifacts
}

dependencies {
    // rustbridge dependencies from Maven Local
    // (Run: cd ../../rustbridge-java && ./gradlew publishToMavenLocal)
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-ffm:0.7.0")

    // JSON serialization
    implementation("com.fasterxml.jackson.core:jackson-databind:2.16.1")

    // Kotlin stdlib
    implementation(kotlin("stdlib"))
}

java {
    toolchain {
        // FFM requires Java 21+
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

kotlin {
    // FFM requires Java 21+
    jvmToolchain(21)
}

// Check if running Java 21 (needs --enable-preview for FFM)
// Java 22+ has FFM as a stable feature and doesn't need this flag
val needsPreview = provider {
    java.toolchain.languageVersion.get().asInt() == 21
}

// Native library path - points to where the Rust library is built
val nativeLibraryPath = "../../target/release"

// Individual example tasks
tasks.register<JavaExec>("runBasic") {
    group = "examples"
    description = "Run basic Kotlin example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.BasicExampleKt")
    systemProperty("java.library.path", nativeLibraryPath)
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

tasks.register<JavaExec>("runLogging") {
    group = "examples"
    description = "Run logging example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.LoggingExampleKt")
    systemProperty("java.library.path", nativeLibraryPath)
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

tasks.register<JavaExec>("runErrorHandling") {
    group = "examples"
    description = "Run error handling example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.ErrorHandlingExampleKt")
    systemProperty("java.library.path", nativeLibraryPath)
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
