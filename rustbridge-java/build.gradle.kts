plugins {
    java
    `java-library`
    `maven-publish`
}

allprojects {
    group = "com.rustbridge"
    version = "0.8.0"

    repositories {
        mavenCentral()
    }
}

subprojects {
    apply(plugin = "java")
    apply(plugin = "java-library")
    apply(plugin = "maven-publish")

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

    // Modules that use FFM APIs directly (rustbridge-core only uses standard Java APIs)
    val usesFfm = project.name != "rustbridge-core"

    tasks.withType<JavaCompile> {
        options.encoding = "UTF-8"
        options.compilerArgs.add("-Xlint:all")
        if (usesFfm && needsPreview.get()) {
            options.compilerArgs.add("--enable-preview")
        }
    }

    tasks.withType<Javadoc> {
        options {
            this as StandardJavadocDocletOptions
            addStringOption("Xdoclint:none", "-quiet")
        }
    }

    tasks.withType<Test> {
        useJUnitPlatform()
        if (usesFfm && needsPreview.get()) {
            jvmArgs("--enable-preview")
        }
    }

    dependencies {
        implementation("org.slf4j:slf4j-api:2.0.9")
        testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
        testRuntimeOnly("org.junit.platform:junit-platform-launcher")
        testRuntimeOnly("org.slf4j:slf4j-simple:2.0.9")
    }

    publishing {
        publications {
            create<MavenPublication>("maven") {
                from(components["java"])
                pom {
                    licenses {
                        license {
                            name.set("MIT License")
                            url.set("https://opensource.org/licenses/MIT")
                        }
                        license {
                            name.set("Apache License, Version 2.0")
                            url.set("https://www.apache.org/licenses/LICENSE-2.0")
                        }
                    }
                }
            }
        }
    }
}
