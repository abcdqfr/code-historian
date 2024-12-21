package io.codehistorian.intellij.service

import com.google.gson.Gson
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.Logger
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.concurrent.TimeUnit

@Service
class HistorianApiService {
    private val logger = Logger.getInstance(HistorianApiService::class.java)
    private val gson = Gson()
    private val client = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(30, TimeUnit.SECONDS)
        .build()

    private val settings = HistorianSettingsService.getInstance()
    private val jsonMediaType = "application/json; charset=utf-8".toMediaType()

    fun startAnalysis(projectPath: String): AnalysisResponse {
        val request = Request.Builder()
            .url("${settings.serverUrl}/api/analysis/start")
            .post(
                gson.toJson(
                    mapOf(
                        "projectPath" to projectPath,
                        "maxDepth" to settings.maxHistoryDepth,
                        "excludedPaths" to settings.excludedPaths
                    )
                ).toRequestBody(jsonMediaType)
            )
            .addHeader("X-API-Key", settings.apiKey)
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Unexpected response ${response.code}")
            }

            return gson.fromJson(
                response.body?.string() ?: throw IOException("Empty response body"),
                AnalysisResponse::class.java
            )
        }
    }

    fun getFileHistory(filePath: String): HistorianProjectService.FileHistory {
        val request = Request.Builder()
            .url("${settings.serverUrl}/api/history/file?path=$filePath")
            .get()
            .addHeader("X-API-Key", settings.apiKey)
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Unexpected response ${response.code}")
            }

            return gson.fromJson(
                response.body?.string() ?: throw IOException("Empty response body"),
                HistorianProjectService.FileHistory::class.java
            )
        }
    }

    fun getProjectMetrics(): ProjectMetrics {
        val request = Request.Builder()
            .url("${settings.serverUrl}/api/metrics/project")
            .get()
            .addHeader("X-API-Key", settings.apiKey)
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Unexpected response ${response.code}")
            }

            return gson.fromJson(
                response.body?.string() ?: throw IOException("Empty response body"),
                ProjectMetrics::class.java
            )
        }
    }

    fun getCustomMetrics(metricKey: String): Map<String, Float> {
        val request = Request.Builder()
            .url("${settings.serverUrl}/api/metrics/custom/$metricKey")
            .get()
            .addHeader("X-API-Key", settings.apiKey)
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Unexpected response ${response.code}")
            }

            return gson.fromJson(
                response.body?.string() ?: throw IOException("Empty response body"),
                Map::class.java
            ) as Map<String, Float>
        }
    }

    data class AnalysisResponse(
        val id: String,
        val status: String,
        val startTime: String
    )

    data class ProjectMetrics(
        val totalFiles: Int,
        val totalCommits: Int,
        val totalAuthors: Int,
        val avgCommitsPerFile: Float,
        val avgAuthorsPerFile: Float,
        val hotspots: List<Hotspot>
    )

    data class Hotspot(
        val filePath: String,
        val score: Float,
        val changes: Int,
        val authors: Int
    )

    companion object {
        fun getInstance(): HistorianApiService = service()
    }
} 