package io.codehistorian.intellij.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.project.Project
import io.codehistorian.intellij.service.HistorianProjectService
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task

class AnalyzeAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val historianService = project.getService(HistorianProjectService::class.java)

        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Analyzing Repository", true) {
            override fun run(indicator: ProgressIndicator) {
                try {
                    indicator.isIndeterminate = false
                    indicator.fraction = 0.0
                    indicator.text = "Starting analysis..."

                    historianService.startAnalysis()

                    showNotification(
                        project,
                        "Analysis Started",
                        "Code history analysis has been started successfully.",
                        NotificationType.INFORMATION
                    )

                } catch (ex: Exception) {
                    showNotification(
                        project,
                        "Analysis Failed",
                        "Failed to start analysis: ${ex.message}",
                        NotificationType.ERROR
                    )
                }
            }
        })
    }

    override fun update(e: AnActionEvent) {
        val project = e.project
        e.presentation.isEnabled = project != null
    }

    private fun showNotification(
        project: Project,
        title: String,
        content: String,
        type: NotificationType
    ) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("Code Historian")
            .createNotification(title, content, type)
            .notify(project)
    }
} 