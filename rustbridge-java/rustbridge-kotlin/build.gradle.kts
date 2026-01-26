plugins {
    `java-library`
    kotlin("jvm") version "1.9.22"
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
    jvmToolchain(21)
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    compilerOptions {
        freeCompilerArgs.add("-Xjsr305=strict")
    }
}

tasks.withType<Test> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}
