plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

rootProject.name = "rustbridge-java"

include("rustbridge-core")
include("rustbridge-ffm")
include("rustbridge-jni")
include("rustbridge-kotlin")
include("rustbridge-benchmarks")
