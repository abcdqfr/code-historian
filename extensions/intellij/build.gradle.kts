plugins {
    id("org.jetbrains.intellij") version "1.13.3"
    id("org.jetbrains.kotlin.jvm") version "1.8.20"
}

group = "io.codehistorian"
version = "1.0.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.squareup.okhttp3:okhttp:4.10.0")
    implementation("com.google.code.gson:gson:2.10.1")
    implementation("org.java-websocket:Java-WebSocket:1.5.3")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-swing:1.7.1")
}

intellij {
    version.set("2023.1")
    type.set("IC") // IntelliJ IDEA Community Edition
    plugins.set(listOf("java"))
}

tasks {
    patchPluginXml {
        sinceBuild.set("231")
        untilBuild.set("241.*")
        
        pluginDescription.set("""
            Code Historian - Advanced Code History Analysis
            
            Features:
            - Real-time code history analysis
            - Impact score visualization
            - Method-level history tracking
            - Team collaboration insights
            - Custom metrics support
            
            This plugin helps you understand your codebase's evolution by:
            - Tracking file and method changes over time
            - Identifying high-impact code areas
            - Visualizing development patterns
            - Supporting team collaboration
            
            For more information, visit: https://codehistorian.io
        """.trimIndent())
        
        changeNotes.set("""
            <h3>1.0.0</h3>
            <ul>
                <li>Initial release</li>
                <li>Real-time analysis support</li>
                <li>Method-level history tracking</li>
                <li>Impact score visualization</li>
                <li>Team collaboration features</li>
            </ul>
        """.trimIndent())
    }

    buildSearchableOptions {
        enabled = false
    }

    compileKotlin {
        kotlinOptions {
            jvmTarget = "17"
            freeCompilerArgs = listOf("-Xjsr305=strict")
        }
    }

    test {
        useJUnitPlatform()
    }
} 