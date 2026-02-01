plugins {
    kotlin("jvm") version "2.3.0"
    application
}

group = "com.example"
version = "1.0.0"

application {
    mainClass.set("com.example.MainKt")
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
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.15.2")

    testImplementation(kotlin("test"))
}

kotlin {
    // JNI works with Java 17+ (LTS versions: 17, 21, 25)
    jvmToolchain(17)
}

tasks.test {
    useJUnitPlatform()
}
