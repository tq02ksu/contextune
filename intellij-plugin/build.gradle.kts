import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.1.0"
    id("org.jetbrains.intellij.platform") version "2.2.1"
}

group = "com.contextune"
version = "0.1.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    compileOnly("net.java.dev.jna:jna:5.13.0")
    compileOnly("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.1")
    testImplementation("org.jetbrains.kotlin:kotlin-test")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
    
    intellijPlatform {
        intellijIdeaCommunity("2025.2.5")
        bundledPlugins(listOf(/* Plugin Dependencies */))
    }
}

kotlin {
    jvmToolchain(21)
}

intellijPlatform {
    buildSearchableOptions = false
    
    instrumentCode = false
}

tasks {
    test {
        useJUnitPlatform()
    }

    patchPluginXml {
        sinceBuild.set("251")
        untilBuild.set("253.*")
    }
    
    runIde {
        jvmArgs = listOf(
            "-Xmx2048m",
            "-XX:+UseG1GC"
        )
        // Disable coroutines debug agent
        systemProperty("kotlinx.coroutines.debug", "off")
    }

    signPlugin {
        certificateChain.set(System.getenv("CERTIFICATE_CHAIN"))
        privateKey.set(System.getenv("PRIVATE_KEY"))
        password.set(System.getenv("PRIVATE_KEY_PASSWORD"))
    }

    publishPlugin {
        token.set(System.getenv("PUBLISH_TOKEN"))
    }
    
    // Copy native libraries to plugin distribution
    prepareSandbox {
        doLast {
            val libsDir = file("libs")
            if (libsDir.exists()) {
                copy {
                    from(libsDir)
                    into("${destinationDir.path}/${pluginName.get()}/lib")
                }
            }
        }
    }
}
