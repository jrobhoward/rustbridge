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
