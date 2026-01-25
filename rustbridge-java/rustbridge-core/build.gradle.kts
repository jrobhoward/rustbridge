plugins {
    `java-library`
}

dependencies {
    api("com.fasterxml.jackson.core:jackson-databind:2.18.2")
    api("com.fasterxml.jackson.module:jackson-module-kotlin:2.18.2")
    implementation("net.i2p.crypto:eddsa:0.3.0") // Ed25519 for Java 7+
}

java {
    withJavadocJar()
    withSourcesJar()
}

// Compile for Java 17 compatibility
tasks.withType<JavaCompile> {
    options.release.set(17)
}
