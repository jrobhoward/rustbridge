plugins {
    kotlin("jvm") version "2.3.0"
    application
}

group = "com.example"
version = "1.0.0"

application {
    mainClass.set("com.example.MainKt")
}

repositories {
    mavenLocal()  // For local rustbridge development
    mavenCentral()
}

dependencies {
    // rustbridge dependencies
    implementation("com.rustbridge:rustbridge-core:0.8.0")
    implementation("com.rustbridge:rustbridge-ffm:0.8.0")

    // JSON serialization
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.15.2")

    testImplementation(kotlin("test"))
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

tasks.withType<org.gradle.api.tasks.compile.JavaCompile> {
    if (needsPreview.get()) {
        options.compilerArgs.add("--enable-preview")
    }
}

tasks.test {
    useJUnitPlatform()
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
}

tasks.withType<JavaExec> {
    // --enable-native-access is always required for FFM
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
