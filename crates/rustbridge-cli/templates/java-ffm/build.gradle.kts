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
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

tasks.test {
    useJUnitPlatform()
}

// Required for FFM preview features
tasks.withType<JavaCompile> {
    options.compilerArgs.add("--enable-preview")
}

tasks.withType<JavaExec> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
