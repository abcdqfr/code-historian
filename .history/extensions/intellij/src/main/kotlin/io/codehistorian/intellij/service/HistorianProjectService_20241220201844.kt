package io.codehistorian.intellij.service

import com.intellij.openapi.components.Service
import com.intellij.openapi.project.Project
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.vfs.VirtualFile
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import org.java_websocket.client.WebSocketClient
import org.java_websocket.handshake.ServerHandshake
import java.net.URI
import com.google.gson.Gson
import com.google.gson.JsonObject

@Service
class HistorianProjectService(private val project: Project) {
    private val logger = Logger.getInstance(HistorianProjectService::class.java)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val gson = Gson()

    private val _analysisState = MutableStateFlow<AnalysisState>(AnalysisState.Idle)
    val analysisState: StateFlow<AnalysisState> = _analysisState

    private var webSocket: WebSocketClient? = null
    private var currentAnalysisId: String? = null

    fun startAnalysis() {
        if (_analysisState.value is AnalysisState.Running) {
            logger.warn("Analysis already running")
            return
        }

        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Analyzing Repository", true) {
            override fun run(indicator: ProgressIndicator) {
                try {
                    indicator.isIndeterminate = false
                    indicator.fraction = 0.0

                    val settings = project.service<HistorianSettingsService>()
                    val apiService = project.service<HistorianApiService>()

                    // Start analysis
                    val response = apiService.startAnalysis(project.basePath!!)
                    currentAnalysisId = response.id

                    // Connect WebSocket
                    connectWebSocket(settings.serverUrl, response.id)

                    _analysisState.value = AnalysisState.Running(0f)

                } catch (e: Exception) {
                    logger.error("Failed to start analysis", e)
                    _analysisState.value = AnalysisState.Error(e.message ?: "Unknown error")
                }
            }
        })
    }

    fun stopAnalysis() {
        webSocket?.close()
        currentAnalysisId = null
        _analysisState.value = AnalysisState.Idle
    }

    private fun connectWebSocket(serverUrl: String, analysisId: String) {
        val wsUrl = serverUrl.replace("http", "ws")
        val uri = URI("$wsUrl/ws/analysis/$analysisId")

        webSocket = object : WebSocketClient(uri) {
            override fun onOpen(handshakedata: ServerHandshake?) {
                logger.info("WebSocket connected")
            }

            override fun onMessage(message: String?) {
                message?.let { handleWebSocketMessage(it) }
            }

            override fun onClose(code: Int, reason: String?, remote: Boolean) {
                logger.info("WebSocket closed: $reason")
                if (_analysisState.value is AnalysisState.Running) {
                    _analysisState.value = AnalysisState.Completed
                }
            }

            override fun onError(ex: Exception?) {
                logger.error("WebSocket error", ex)
                _analysisState.value = AnalysisState.Error(ex?.message ?: "Unknown error")
            }
        }

        webSocket?.connect()
    }

    private fun handleWebSocketMessage(message: String) {
        scope.launch {
            try {
                val json = gson.fromJson(message, JsonObject::class.java)
                
                when {
                    json.has("progress") -> {
                        val progress = json.get("progress").asFloat
                        _analysisState.value = AnalysisState.Running(progress)
                    }
                    json.has("error") -> {
                        val error = json.get("error").asString
                        _analysisState.value = AnalysisState.Error(error)
                    }
                    json.has("completed") -> {
                        _analysisState.value = AnalysisState.Completed
                    }
                }

            } catch (e: Exception) {
                logger.error("Failed to parse WebSocket message", e)
            }
        }
    }

    fun getFileHistory(file: VirtualFile): FileHistory? {
        return try {
            val apiService = project.service<HistorianApiService>()
            apiService.getFileHistory(file.path)
        } catch (e: Exception) {
            logger.error("Failed to get file history", e)
            null
        }
    }

    sealed class AnalysisState {
        object Idle : AnalysisState()
        data class Running(val progress: Float) : AnalysisState()
        object Completed : AnalysisState()
        data class Error(val message: String) : AnalysisState()
    }

    data class FileHistory(
        val changes: List<Change>,
        val metrics: Metrics
    )

    data class Change(
        val timestamp: String,
        val author: String,
        val message: String,
        val impactScore: Float
    )

    data class Metrics(
        val totalChanges: Int,
        val totalAuthors: Int,
        val avgImpactScore: Float
    )

    companion object {
        fun getInstance(project: Project): HistorianProjectService = project.service()
    }
} 