package io.codehistorian.intellij.window

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.dsl.builder.*
import com.intellij.ui.components.JBTabbedPane
import io.codehistorian.intellij.service.HistorianProjectService
import javax.swing.JPanel
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.table.JBTable
import javax.swing.table.DefaultTableModel
import java.awt.BorderLayout
import javax.swing.JButton
import javax.swing.SwingUtilities

class HistorianToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val contentFactory = ContentFactory.getInstance()
        val content = contentFactory.createContent(createMainPanel(project), "", false)
        toolWindow.contentManager.addContent(content)
    }

    private fun createMainPanel(project: Project): JPanel {
        val mainPanel = JPanel(BorderLayout())
        val tabbedPane = JBTabbedPane()

        // Dashboard Tab
        tabbedPane.addTab("Dashboard", createDashboardPanel(project))

        // File History Tab
        tabbedPane.addTab("File History", createFileHistoryPanel(project))

        // Project Metrics Tab
        tabbedPane.addTab("Project Metrics", createProjectMetricsPanel(project))

        mainPanel.add(tabbedPane, BorderLayout.CENTER)
        return mainPanel
    }

    private fun createDashboardPanel(project: Project): JPanel {
        val panel = JPanel(BorderLayout())
        val historianService = project.getService(HistorianProjectService::class.java)

        // Status Section
        val statusPanel = panel {
            row {
                label("Analysis Status:")
                    .bold()
            }
            row {
                val statusLabel = label("")
                    .component

                historianService.scope.launch {
                    historianService.analysisState.collectLatest { state ->
                        SwingUtilities.invokeLater {
                            statusLabel.text = when (state) {
                                is HistorianProjectService.AnalysisState.Idle -> "Idle"
                                is HistorianProjectService.AnalysisState.Running -> 
                                    "Running (${(state.progress * 100).toInt()}%)"
                                is HistorianProjectService.AnalysisState.Completed -> "Completed"
                                is HistorianProjectService.AnalysisState.Error -> "Error: ${state.message}"
                            }
                        }
                    }
                }
            }
        }

        // Controls Section
        val controlsPanel = panel {
            row {
                button("Start Analysis") {
                    historianService.startAnalysis()
                }
                button("Stop Analysis") {
                    historianService.stopAnalysis()
                }
            }
        }

        panel.add(statusPanel.component, BorderLayout.NORTH)
        panel.add(controlsPanel.component, BorderLayout.CENTER)
        return panel
    }

    private fun createFileHistoryPanel(project: Project): JPanel {
        val panel = JPanel(BorderLayout())
        val historianService = project.getService(HistorianProjectService::class.java)

        val tableModel = DefaultTableModel(
            arrayOf("Timestamp", "Author", "Message", "Impact Score"),
            0
        )
        val table = JBTable(tableModel)
        
        // Add table to scrollpane
        panel.add(JBScrollPane(table), BorderLayout.CENTER)

        // Add refresh button
        val refreshButton = JButton("Refresh").apply {
            addActionListener {
                val selectedFile = project.selectedFile
                if (selectedFile != null) {
                    val history = historianService.getFileHistory(selectedFile)
                    tableModel.rowCount = 0
                    history?.changes?.forEach { change ->
                        tableModel.addRow(arrayOf(
                            change.timestamp,
                            change.author,
                            change.message,
                            change.impactScore
                        ))
                    }
                }
            }
        }
        panel.add(refreshButton, BorderLayout.NORTH)

        return panel
    }

    private fun createProjectMetricsPanel(project: Project): JPanel {
        val panel = JPanel(BorderLayout())
        val historianService = project.getService(HistorianProjectService::class.java)

        // Metrics Overview Section
        val metricsPanel = panel {
            group("Overview") {
                row("Total Files:") {
                    label("0")
                }
                row("Total Commits:") {
                    label("0")
                }
                row("Total Authors:") {
                    label("0")
                }
            }

            group("Hotspots") {
                val tableModel = DefaultTableModel(
                    arrayOf("File", "Score", "Changes", "Authors"),
                    0
                )
                row {
                    scrollPane(JBTable(tableModel))
                        .resizableColumn()
                }
            }
        }

        // Add refresh button
        val refreshButton = JButton("Refresh").apply {
            addActionListener {
                try {
                    val apiService = project.getService(HistorianApiService::class.java)
                    val metrics = apiService.getProjectMetrics()
                    // Update metrics display
                    // TODO: Implement metrics update logic
                } catch (e: Exception) {
                    // Handle error
                }
            }
        }

        panel.add(refreshButton, BorderLayout.NORTH)
        panel.add(metricsPanel.component, BorderLayout.CENTER)
        return panel
    }

    private val Project.selectedFile
        get() = null // TODO: Implement selected file detection
} 