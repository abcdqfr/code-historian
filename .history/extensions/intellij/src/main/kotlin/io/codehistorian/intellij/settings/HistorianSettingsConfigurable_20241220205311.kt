package io.codehistorian.intellij.settings

import com.intellij.openapi.options.Configurable
import com.intellij.openapi.ui.DialogPanel
import com.intellij.ui.components.JBCheckBox
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBTextField
import com.intellij.ui.dsl.builder.*
import com.intellij.util.ui.FormBuilder
import io.codehistorian.intellij.service.HistorianSettingsService
import javax.swing.JComponent
import javax.swing.JPanel

class HistorianSettingsConfigurable : Configurable {
    private val settings = HistorianSettingsService.getInstance()
    private var mainPanel: DialogPanel? = null

    private var serverUrlField: JBTextField? = null
    private var apiKeyField: JBTextField? = null
    private var enableNotificationsCheckbox: JBCheckBox? = null
    private var enableLineMarkersCheckbox: JBCheckBox? = null
    private var enableAutoAnalysisCheckbox: JBCheckBox? = null
    private var maxHistoryDepthField: JBTextField? = null

    override fun createComponent(): JComponent {
        mainPanel = panel {
            group("Server Settings") {
                row("Server URL:") {
                    serverUrlField = textField()
                        .text(settings.serverUrl)
                        .focused()
                        .component
                }
                row("API Key:") {
                    apiKeyField = textField()
                        .text(settings.apiKey)
                        .component
                }
            }

            group("Analysis Settings") {
                row {
                    enableNotificationsCheckbox = checkBox("Enable Notifications")
                        .selected(settings.enableNotifications)
                        .component
                }
                row {
                    enableLineMarkersCheckbox = checkBox("Enable Line Markers")
                        .selected(settings.enableLineMarkers)
                        .component
                }
                row {
                    enableAutoAnalysisCheckbox = checkBox("Enable Auto Analysis")
                        .selected(settings.enableAutoAnalysis)
                        .component
                }
                row("Max History Depth:") {
                    maxHistoryDepthField = textField()
                        .text(settings.maxHistoryDepth.toString())
                        .component
                }
            }

            group("Excluded Paths") {
                row {
                    textArea()
                        .text(settings.excludedPaths.joinToString("\n"))
                        .rows(5)
                        .resizableColumn()
                        .comment("One path per line. Use glob patterns (e.g. *.txt, test/**, etc.)")
                }
            }

            group("Custom Metrics") {
                row {
                    textArea()
                        .text(settings.customMetrics.entries.joinToString("\n") { "${it.key}=${it.value}" })
                        .rows(5)
                        .resizableColumn()
                        .comment("One metric per line. Format: name=expression")
                }
            }
        }

        return mainPanel!!
    }

    override fun isModified(): Boolean {
        return serverUrlField?.text != settings.serverUrl ||
                apiKeyField?.text != settings.apiKey ||
                enableNotificationsCheckbox?.isSelected != settings.enableNotifications ||
                enableLineMarkersCheckbox?.isSelected != settings.enableLineMarkers ||
                enableAutoAnalysisCheckbox?.isSelected != settings.enableAutoAnalysis ||
                maxHistoryDepthField?.text?.toIntOrNull() != settings.maxHistoryDepth
    }

    override fun apply() {
        settings.serverUrl = serverUrlField?.text ?: settings.serverUrl
        settings.apiKey = apiKeyField?.text ?: settings.apiKey
        settings.enableNotifications = enableNotificationsCheckbox?.isSelected ?: settings.enableNotifications
        settings.enableLineMarkers = enableLineMarkersCheckbox?.isSelected ?: settings.enableLineMarkers
        settings.enableAutoAnalysis = enableAutoAnalysisCheckbox?.isSelected ?: settings.enableAutoAnalysis
        settings.maxHistoryDepth = maxHistoryDepthField?.text?.toIntOrNull() ?: settings.maxHistoryDepth
    }

    override fun getDisplayName(): String = "Code Historian"

    override fun reset() {
        serverUrlField?.text = settings.serverUrl
        apiKeyField?.text = settings.apiKey
        enableNotificationsCheckbox?.isSelected = settings.enableNotifications
        enableLineMarkersCheckbox?.isSelected = settings.enableLineMarkers
        enableAutoAnalysisCheckbox?.isSelected = settings.enableAutoAnalysis
        maxHistoryDepthField?.text = settings.maxHistoryDepth.toString()
    }
} 