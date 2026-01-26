plugins {
    java
    id("me.champeau.jmh") version "0.7.2"
}

dependencies {
    // Project dependencies
    implementation(project(":rustbridge-core"))
    implementation(project(":rustbridge-ffm"))
    implementation(project(":rustbridge-jni"))

    // JMH
    jmh("org.openjdk.jmh:jmh-core:1.37")
    jmhAnnotationProcessor("org.openjdk.jmh:jmh-generator-annprocess:1.37")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

// Enable preview features for compilation (needed for Arena/MemorySegment)
tasks.withType<JavaCompile> {
    options.compilerArgs.addAll(listOf("--enable-preview"))
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
    jvmArgs.set(listOf(
        "--enable-preview",
        "--enable-native-access=ALL-UNNAMED",
        "-Djava.library.path=${rootProject.projectDir.parentFile}/target/release"
    ))
}

// Configure bytecode generator task to use preview features when it runs
afterEvaluate {
    tasks.named<me.champeau.jmh.JmhBytecodeGeneratorTask>("jmhRunBytecodeGenerator") {
        jvmArgs.add("--enable-preview")
    }
}
