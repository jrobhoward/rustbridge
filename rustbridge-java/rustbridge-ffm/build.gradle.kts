plugins {
    `java-library`
}

dependencies {
    api(project(":rustbridge-core"))
    testImplementation(project(":rustbridge-jni"))
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

    // Set native library path to find rustbridge_jni and hello_plugin (for benchmark comparison tests)
    val rustTargetDir = rootProject.projectDir.parentFile.resolve("target")
    val releaseDir = rustTargetDir.resolve("release")
    val debugDir = rustTargetDir.resolve("debug")

    val libraryPath = if (releaseDir.resolve("librustbridge_jni.so").exists() ||
        releaseDir.resolve("librustbridge_jni.dylib").exists() ||
        releaseDir.resolve("rustbridge_jni.dll").exists()
    ) {
        releaseDir.absolutePath
    } else {
        debugDir.absolutePath
    }

    systemProperty("java.library.path", libraryPath)
}

tasks.withType<Javadoc> {
    val opts = options as StandardJavadocDocletOptions
    opts.addBooleanOption("-enable-preview", true)
    opts.addStringOption("source", "21")
}
