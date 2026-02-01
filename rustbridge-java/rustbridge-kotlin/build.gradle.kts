plugins {
    `java-library`
    kotlin("jvm") version "2.0.21"
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
    // Java 22+ required for FFM
    jvmToolchain(22)
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    compilerOptions {
        freeCompilerArgs.add("-Xjsr305=strict")
    }
}

tasks.withType<Test> {
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}
