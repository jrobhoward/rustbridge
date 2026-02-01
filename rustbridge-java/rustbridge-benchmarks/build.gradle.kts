plugins {
    java
    id("me.champeau.jmh") version "0.7.2"
}

dependencies {
    // Project dependencies
    implementation(project(":rustbridge-core"))
    implementation(project(":rustbridge-ffm"))

    // JMH
    jmh("org.openjdk.jmh:jmh-core:1.37")
    jmhAnnotationProcessor("org.openjdk.jmh:jmh-generator-annprocess:1.37")
}

java {
    toolchain {
        // FFM requires Java 21+
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

// Check if running Java 21 (needs --enable-preview for FFM)
val needsPreview = provider {
    java.toolchain.languageVersion.get().asInt() == 21
}

jmh {
    // JMH configuration
    warmupIterations.set(3)
    iterations.set(5)
    fork.set(2)
    threads.set(1)

    // Output format
    resultFormat.set("JSON")
    resultsFile.set(project.file("build/reports/jmh/results.json"))

    // Include all benchmarks by default
    includes.set(listOf(".*Benchmark.*"))

    // JVM args for benchmark execution
    // --enable-native-access is always required for FFM
    // --enable-preview is only needed for Java 21 (FFM is stable in Java 22+)
    jvmArgs.set(buildList {
        if (needsPreview.get()) {
            add("--enable-preview")
        }
        add("--enable-native-access=ALL-UNNAMED")
        add("-Djava.library.path=${rootProject.projectDir.parentFile}/target/release")
    })
}
