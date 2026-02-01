plugins {
    `java-library`
}

// Java 22+ required for FFM
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(22))
    }
    withJavadocJar()
    withSourcesJar()
}

dependencies {
    api(project(":rustbridge-core"))
}

tasks.withType<Test> {
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}

tasks.withType<Javadoc> {
    val opts = options as StandardJavadocDocletOptions
    opts.addStringOption("source", "22")
}
