package io.codehistorian.intellij.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.popup.JBPopupFactory
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.table.JBTable
import io.codehistorian.intellij.service.HistorianProjectService
import java.awt.Dimension
import javax.swing.table.DefaultTableModel

class ShowHistoryAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val editor = e.getData(CommonDataKeys.EDITOR) ?: return
        val file = e.getData(CommonDataKeys.VIRTUAL_FILE) ?: return
        val historianService = project.getService(HistorianProjectService::class.java)

        try {
            val history = historianService.getFileHistory(file)
            if (history == null) {
                showErrorPopup(project, "No history available for this file")
                return
            }

            showHistoryPopup(project, history)

        } catch (ex: Exception) {
            showErrorPopup(project, "Failed to load history: ${ex.message}")
        }
    }

    override fun update(e: AnActionEvent) {
        val project = e.project
        val editor = e.getData(CommonDataKeys.EDITOR)
        val file = e.getData(CommonDataKeys.VIRTUAL_FILE)
        e.presentation.isEnabled = project != null && editor != null && file != null
    }

    private fun showHistoryPopup(project: Project, history: HistorianProjectService.FileHistory) {
        val tableModel = DefaultTableModel(
            arrayOf("Timestamp", "Author", "Message", "Impact Score"),
            0
        )

        history.changes.forEach { change ->
            tableModel.addRow(arrayOf(
                change.timestamp,
                change.author,
                change.message,
                String.format("%.2f", change.impactScore)
            ))
        }

        val table = JBTable(tableModel).apply {
            setShowGrid(true)
            autoResizeMode = JBTable.AUTO_RESIZE_ALL_COLUMNS
        }

        val scrollPane = JBScrollPane(table)
        scrollPane.preferredSize = Dimension(800, 400)

        JBPopupFactory.getInstance()
            .createComponentPopupBuilder(scrollPane, null)
            .setTitle("File History")
            .setMovable(true)
            .setResizable(true)
            .createPopup()
            .showCenteredInCurrentWindow(project)
    }

    private fun showErrorPopup(project: Project, message: String) {
        JBPopupFactory.getInstance()
            .createMessage(message)
            .showCenteredInCurrentWindow(project)
    }
} 