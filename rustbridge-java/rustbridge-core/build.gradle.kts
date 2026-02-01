plugins {
    `java-library`
}

dependencies {
    api("com.fasterxml.jackson.core:jackson-databind:2.18.2")
    api("com.fasterxml.jackson.module:jackson-module-kotlin:2.18.2")
    api("org.jetbrains:annotations:26.0.1") // Null-safety annotations
    implementation("net.i2p.crypto:eddsa:0.3.0") // Ed25519 for Java 7+
    implementation("org.bouncycastle:bcprov-jdk18on:1.78.1") // BLAKE2b for minisign prehashing
}

java {
    withJavadocJar()
    withSourcesJar()
}

// Compile for Java 21 (FFM is the only implementation, requires 21+)
tasks.withType<JavaCompile> {
    options.release.set(21)
}
