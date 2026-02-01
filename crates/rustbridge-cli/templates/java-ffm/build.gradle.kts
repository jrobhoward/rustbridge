plugins {
    java
    application
}

group = "com.example"
version = "1.0.0"

application {
    mainClass.set("com.example.Main")
}

repositories {
    mavenLocal()  // For local rustbridge development
    mavenCentral()
}

dependencies {
    // rustbridge dependencies
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-ffm:0.7.0")

    // JSON serialization
    implementation("com.google.code.gson:gson:2.10.1")

    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
}

java {
    toolchain {
        // FFM requires Java 21+
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

// Check if running Java 21 (needs --enable-preview for FFM)
// Java 22+ has FFM as a stable feature and doesn't need this flag
val needsPreview = provider {
    java.toolchain.languageVersion.get().asInt() == 21
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<JavaCompile> {
    if (needsPreview.get()) {
        options.compilerArgs.add("--enable-preview")
    }
}

tasks.withType<JavaExec> {
    // --enable-native-access is always required for FFM
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}

tasks.withType<Test> {
    if (needsPreview.get()) {
        jvmArgs("--enable-preview")
    }
}
