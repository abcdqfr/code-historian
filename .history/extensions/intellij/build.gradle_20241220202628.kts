plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "1.8.0"
    id("org.jetbrains.intellij") version "1.13.3"
}

group = "io.codehistorian"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.squareup.retrofit2:retrofit:2.9.0")
    implementation("com.squareup.retrofit2:converter-gson:2.9.0")
    implementation("org.java-websocket:Java-WebSocket:1.5.3")
    implementation("com.google.code.gson:gson:2.10.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-swing:1.7.1")
    
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testImplementation("io.mockk:mockk:1.13.4")
}

intellij {
    version.set("2023.1")
    type.set("IC") // IntelliJ IDEA Community Edition
    plugins.set(listOf("java"))
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
        sinceBuild.set("231")
        untilBuild.set("233.*")
        
        pluginDescription.set("""
            Code Historian plugin for IntelliJ IDEA.
            
            Features:
            - Real-time code evolution analysis
            - Interactive visualizations
            - Team collaboration insights
            - Project comparison tools
            - Impact analysis
            
            Integrates seamlessly with the Code Historian server to provide insights about your codebase's development patterns.
        """.trimIndent())
        
        changeNotes.set("""
            Initial release:
            - Real-time dashboard
            - Analysis progress tracking
            - Team activity monitoring
            - Project metrics visualization
            - Code impact analysis
        """.trimIndent())
    }

    runIde {
        autoReloadPlugins.set(true)
    }

    test {
        useJUnitPlatform()
    }
} 