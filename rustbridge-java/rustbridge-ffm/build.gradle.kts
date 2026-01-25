plugins {
    `java-library`
}

dependencies {
    api(project(":rustbridge-core"))
}

java {
    withJavadocJar()
    withSourcesJar()
}

tasks.withType<JavaCompile> {
    options.compilerArgs.addAll(
        listOf(
            "--enable-preview"
        )
    )
}

tasks.withType<Test> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
    // Default timeout for all tests - prevents builds from hanging indefinitely
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}

tasks.withType<Javadoc> {
    val opts = options as StandardJavadocDocletOptions
    opts.addBooleanOption("-enable-preview", true)
    opts.addStringOption("source", "21")
}
