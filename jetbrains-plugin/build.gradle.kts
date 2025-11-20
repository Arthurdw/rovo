plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.1.0"  // Latest stable compatible with IntelliJ plugin
    id("org.jetbrains.intellij") version "1.17.4"   // Latest 1.x version
}

group = "com.rovo"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib")
}

intellij {
    version.set("2024.3")  // Latest stable IntelliJ Platform
    type.set("IC") // IntelliJ IDEA Community Edition
    plugins.set(listOf("com.redhat.devtools.lsp4ij:0.10.0"))  // Updated LSP4IJ
}

tasks {
    withType<JavaCompile> {
        sourceCompatibility = "17"
        targetCompatibility = "17"
    }

    withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
        kotlinOptions.jvmTarget = "17"
    }

    patchPluginXml {
        sinceBuild.set("243")  // 2024.3+
        untilBuild.set("999.*")  // Support all future versions
    }

    signPlugin {
        certificateChain.set(System.getenv("CERTIFICATE_CHAIN"))
        privateKey.set(System.getenv("PRIVATE_KEY"))
        password.set(System.getenv("PRIVATE_KEY_PASSWORD"))
    }

    publishPlugin {
        token.set(System.getenv("PUBLISH_TOKEN"))
    }
}
