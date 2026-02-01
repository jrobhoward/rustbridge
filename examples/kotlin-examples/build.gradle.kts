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
    implementation("com.rustbridge:rustbridge-jni:0.7.0")  // JNI is recommended for all Java versions

    // JSON serialization
    implementation("com.fasterxml.jackson.core:jackson-databind:2.16.1")

    // Kotlin stdlib
    implementation(kotlin("stdlib"))
}

java {
    toolchain {
        // Java 17+ for JNI (recommended)
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

kotlin {
    // Java 17+ for JNI (recommended)
    jvmToolchain(17)
}

// JNI library path - points to where librustbridge_jni.so is built
val jniLibraryPath = "../../target/release"

// Individual example tasks
tasks.register<JavaExec>("runBasic") {
    group = "examples"
    description = "Run basic Kotlin example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.BasicExampleKt")
    systemProperty("java.library.path", jniLibraryPath)
}

tasks.register<JavaExec>("runLogging") {
    group = "examples"
    description = "Run logging example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.LoggingExampleKt")
    systemProperty("java.library.path", jniLibraryPath)
}

tasks.register<JavaExec>("runErrorHandling") {
    group = "examples"
    description = "Run error handling example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.ErrorHandlingExampleKt")
    systemProperty("java.library.path", jniLibraryPath)
}
