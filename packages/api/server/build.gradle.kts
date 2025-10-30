import io.ktor.plugin.OpenApiPreview

val kotlin_version: String by project
val logback_version: String by project

plugins {
    kotlin("jvm") version "2.2.20"
    id("io.ktor.plugin") version "3.3.1"
    id("org.jetbrains.kotlin.plugin.serialization") version "2.2.20"
}

application {
    mainClass = "io.t8n.kedeng.ApplicationKt"
}

kotlin {
    jvmToolchain(24)
    compilerOptions {
        optIn.add("kotlin.uuid.ExperimentalUuidApi")
        optIn.add("kotlin.time.ExperimentalTime")
    }
}

ktor {
    docker {
        localImageName.set("kedeng-api")
        jreVersion = JavaVersion.VERSION_24
    }

    @OptIn(OpenApiPreview::class)
    openApi {
        title = "kedeng"
        version = "v1"

        target = project.layout.buildDirectory.file("openapi.json")
    }
}

dependencies {
    implementation("io.ktor:ktor-server-cors")
    implementation("io.ktor:ktor-server-core")
    implementation("io.ktor:ktor-server-host-common")
    implementation("io.ktor:ktor-server-status-pages")
    implementation("io.ktor:ktor-server-content-negotiation")
    implementation("io.ktor:ktor-serialization-kotlinx-json")
    implementation("io.ktor:ktor-server-netty")
    implementation("io.ktor:ktor-server-di")
    implementation("io.ktor:ktor-server-sse")
    implementation("ch.qos.logback:logback-classic:$logback_version")
    implementation("io.ktor:ktor-server-config-yaml")
    implementation("com.influxdb:influxdb-client-kotlin:7.3.0")
    implementation("com.influxdb:flux-dsl:7.3.0")

    implementation("dev.hayden:khealth:3.0.2")

    implementation(libs.exposed.core)
    implementation(libs.exposed.jdbc)
    implementation(libs.exposed.dao)
    implementation(libs.exposed.kotlin.datetime)
    implementation("org.postgresql:postgresql:42.7.8")

    implementation("de.stuebingerb:kgraphql:0.35.0")
    implementation("de.stuebingerb:kgraphql-ktor:0.35.0")

    testImplementation("io.ktor:ktor-server-test-host")
    testImplementation("org.jetbrains.kotlin:kotlin-test-junit:$kotlin_version")

    api("io.opentelemetry:opentelemetry-sdk-extension-autoconfigure:1.52.0")
    api("io.opentelemetry.semconv:opentelemetry-semconv:1.34.0")
    api("io.opentelemetry:opentelemetry-exporter-otlp:1.52.0")
    api("io.opentelemetry.instrumentation:opentelemetry-ktor-3.0:2.18.0-alpha")
}

repositories {
    mavenCentral()
    maven {
        url = uri("https://jitpack.io")
        name = "jitpack"
    }
}
