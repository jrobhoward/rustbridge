plugins {
    `java-library`
}

dependencies {
    api(project(":rustbridge-core"))
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
    withJavadocJar()
    withSourcesJar()
}

// Compile for Java 8 compatibility using Java 21 compiler
tasks.withType<JavaCompile> {
    options.release.set(8)
}

tasks.withType<Test> {
    // Default timeout for all tests - prevents builds from hanging indefinitely
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}
