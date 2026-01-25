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

// Compile for Java 17 compatibility
tasks.withType<JavaCompile> {
    options.release.set(17)
}

tasks.withType<Test> {
    // Default timeout for all tests - prevents builds from hanging indefinitely
    systemProperty("junit.jupiter.execution.timeout.default", "60s")

    // Set native library path to find rustbridge_jni and hello_plugin
    // Try release first, fall back to debug
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

    // Log where we're looking for libraries
    doFirst {
        println("Using native library path: $libraryPath")
    }
}
