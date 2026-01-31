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
    // rustbridge dependencies (JNI for Java 17+ compatibility)
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-jni:0.7.0")

    // JSON serialization
    implementation("com.google.code.gson:gson:2.10.1")

    testImplementation("org.junit.jupiter:junit-jupiter:5.10.0")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))  // JNI requires Java 17+ (LTS)
    }
}

tasks.test {
    useJUnitPlatform()
}

// Set library path for JNI native library
tasks.withType<JavaExec> {
    // Update this path to where librustbridge_jni.so is located
    systemProperty("java.library.path", System.getProperty("java.library.path", "") +
        ":../../target/release")
}
