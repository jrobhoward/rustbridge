plugins {
    kotlin("jvm") version "1.9.22"
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
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

kotlin {
    jvmToolchain(21)
}

// Individual example tasks
tasks.register<JavaExec>("runBasic") {
    group = "examples"
    description = "Run basic Kotlin example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.BasicExampleKt")
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}

tasks.register<JavaExec>("runLogging") {
    group = "examples"
    description = "Run logging example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.LoggingExampleKt")
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}

tasks.register<JavaExec>("runErrorHandling") {
    group = "examples"
    description = "Run error handling example"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("com.rustbridge.examples.ErrorHandlingExampleKt")
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
