plugins {
    `java-library`
}

// Multi-release JAR: supports Java 21+ (preview) and Java 22+ (stable FFM APIs)
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
    withJavadocJar()
    withSourcesJar()
}

dependencies {
    api(project(":rustbridge-core"))
}

// Base classes use Java 21 preview APIs (allocateUtf8String, getUtf8String)
tasks.withType<JavaCompile>().configureEach {
    if (name == "compileJava") {
        options.compilerArgs.add("--enable-preview")
    }
}

// Java 22+ source set with stable APIs (allocateFrom, getString)
sourceSets {
    create("java22") {
        java.srcDir("src/main/java22")
    }
}

// Compile java22 sources with Java 22 toolchain (no preview needed)
// Include main's output so java22 can see NativeBindings, etc.
tasks.named<JavaCompile>("compileJava22Java") {
    dependsOn(tasks.compileJava)
    classpath = sourceSets.main.get().compileClasspath + sourceSets.main.get().output
    javaCompiler.set(javaToolchains.compilerFor {
        languageVersion.set(JavaLanguageVersion.of(22))
    })
}

// Package as multi-release JAR
tasks.jar {
    dependsOn("compileJava22Java")
    into("META-INF/versions/22") {
        from(sourceSets["java22"].output)
    }
    manifest {
        attributes("Multi-Release" to "true")
    }
}

// Tests need both preview (Java 21) and native access flags
tasks.withType<Test> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
    systemProperty("junit.jupiter.execution.timeout.default", "60s")
}

tasks.withType<Javadoc> {
    val opts = options as StandardJavadocDocletOptions
    opts.addStringOption("source", "21")
    opts.addBooleanOption("-enable-preview", true)
}
