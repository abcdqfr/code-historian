package io.codehistorian.intellij.service

import com.intellij.openapi.components.*
import com.intellij.util.xmlb.XmlSerializerUtil
import com.intellij.credentialStore.CredentialAttributes
import com.intellij.credentialStore.PasswordSafe

@State(
    name = "HistorianSettings",
    storageClass = StateStorage::class,
    secure = true
)
@Service
class HistorianSettingsService : PersistentStateComponent<HistorianSettingsService.State> {
    var serverUrl: String = "http://localhost:3000"
    var apiKey: String = ""
        get() = PasswordSafe.instance.getPassword(CREDENTIALS_ATTR, PROJECT_ID) ?: ""
        set(value) {
            PasswordSafe.instance.setPassword(CREDENTIALS_ATTR, PROJECT_ID, value)
            field = ""  // Don't store in memory
        }
    var enableNotifications: Boolean = true
    var enableLineMarkers: Boolean = true
    var enableAutoAnalysis: Boolean = false
    var maxHistoryDepth: Int = 100
    var excludedPaths: List<String> = listOf()
    var customMetrics: Map<String, String> = mapOf()

    override fun getState(): HistorianSettingsService.State = this

    override fun loadState(state: HistorianSettingsService.State) {
        XmlSerializerUtil.copyBean(state, this)
    }

    companion object {
        fun getInstance(): HistorianSettingsService = service()
        private const val PROJECT_ID = "CodeHistorian"
        private val CREDENTIALS_ATTR = CredentialAttributes(PROJECT_ID)
    }
} 