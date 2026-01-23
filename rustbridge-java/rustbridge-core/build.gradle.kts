plugins {
    `java-library`
}

dependencies {
    api("com.google.code.gson:gson:2.10.1")
}

java {
    withJavadocJar()
    withSourcesJar()
}

// Compile for Java 8 compatibility (for JNI support)
tasks.withType<JavaCompile> {
    options.release.set(8)
}
