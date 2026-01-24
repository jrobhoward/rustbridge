plugins {
    `java-library`
}

dependencies {
    api("com.google.code.gson:gson:2.10.1")
    implementation("net.i2p.crypto:eddsa:0.3.0") // Ed25519 for Java 7+
}

java {
    withJavadocJar()
    withSourcesJar()
}

// Compile for Java 8 compatibility (for JNI support)
tasks.withType<JavaCompile> {
    options.release.set(8)
}
