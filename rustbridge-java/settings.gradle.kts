plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "0.9.0"
}

rootProject.name = "rustbridge-java"

include("rustbridge-core")
include("rustbridge-ffm")
include("rustbridge-jni")
include("rustbridge-kotlin")
include("rustbridge-benchmarks")
