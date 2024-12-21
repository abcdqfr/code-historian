package io.codehistorian.intellij.service

import com.intellij.openapi.components.*
import com.intellij.util.xmlb.XmlSerializerUtil

@State(
    name = "HistorianSettings",
    storages = [Storage("historian.xml")]
)
@Service
class HistorianSettingsService : PersistentStateComponent<HistorianSettingsService> {
    var serverUrl: String = "http://localhost:8080"
    var apiKey: String = ""
    var enableNotifications: Boolean = true
    var enableLineMarkers: Boolean = true
    var enableAutoAnalysis: Boolean = false
    var maxHistoryDepth: Int = 100
    var excludedPaths: List<String> = listOf()
    var customMetrics: Map<String, String> = mapOf()

    override fun getState(): HistorianSettingsService = this

    override fun loadState(state: HistorianSettingsService) {
        XmlSerializerUtil.copyBean(state, this)
    }

    companion object {
        fun getInstance(): HistorianSettingsService = service()
    }
} 