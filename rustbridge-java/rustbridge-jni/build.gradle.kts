plugins {
    `java-library`
}

dependencies {
    api(project(":rustbridge-core"))
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(8))
    }
    withJavadocJar()
    withSourcesJar()
}
