plugins {
    `java-library`
    kotlin("jvm") version "2.3.0"
}

dependencies {
    api(project(":rustbridge-core"))
    api(project(":rustbridge-ffm"))

    // Kotlin stdlib
    implementation(kotlin("stdlib"))

    // Coroutines
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.0")

    // JSON serialization (reuse Jackson with Kotlin module)
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.18.2")

    // Test dependencies
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.8.0")
}

java {
    withJavadocJar()
    withSourcesJar()
}

kotlin {
    // FFM requires Java 21+
    jvmToolchain(21)
}

// Check if running Java 21 (needs --enable-preview for FFM)
val needsPreview = provider {
    java.toolchain.languageVersion.get().asInt() == 21
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    compilerOptions {
        freeCompilerArgs.add("-Xjsr305=strict")
    }
}

tasks.withType<Test> {
    // --enable-native-access is always required for FFM
    // --enable-preview is only needed for Java 21 (FFM is stable in Java 22+)
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}
