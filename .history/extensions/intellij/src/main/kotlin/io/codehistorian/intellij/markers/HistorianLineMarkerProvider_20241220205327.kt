package io.codehistorian.intellij.markers

import com.intellij.codeInsight.daemon.LineMarkerInfo
import com.intellij.codeInsight.daemon.LineMarkerProvider
import com.intellij.openapi.editor.markup.GutterIconRenderer
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiMethod
import com.intellij.ui.JBColor
import io.codehistorian.intellij.service.HistorianProjectService
import io.codehistorian.intellij.service.HistorianSettingsService
import java.awt.Color
import javax.swing.Icon
import com.intellij.icons.AllIcons
import com.intellij.openapi.editor.markup.RangeHighlighter
import com.intellij.openapi.editor.markup.TextAttributes
import java.awt.Font

class HistorianLineMarkerProvider : LineMarkerProvider {
    override fun getLineMarkerInfo(element: PsiElement): LineMarkerInfo<*>? {
        if (!HistorianSettingsService.getInstance().enableLineMarkers) {
            return null
        }

        if (element !is PsiMethod) {
            return null
        }

        val project = element.project
        val historianService = project.getService(HistorianProjectService::class.java)

        val file = element.containingFile.virtualFile ?: return null
        val history = historianService.getFileHistory(file) ?: return null

        // Calculate method impact score
        val impactScore = calculateMethodImpactScore(history)
        if (impactScore <= 0) {
            return null
        }

        return createLineMarkerInfo(element, impactScore)
    }

    private fun calculateMethodImpactScore(history: HistorianProjectService.FileHistory): Float {
        // Calculate impact score based on:
        // 1. Number of changes
        // 2. Number of different authors
        // 3. Average impact score of changes
        val changes = history.changes.size
        val authors = history.changes.map { it.author }.distinct().size
        val avgImpact = history.changes.map { it.impactScore }.average().toFloat()

        return (changes * 0.4f + authors * 0.3f + avgImpact * 0.3f).coerceIn(0f, 1f)
    }

    private fun createLineMarkerInfo(
        element: PsiElement,
        impactScore: Float
    ): LineMarkerInfo<PsiElement> {
        val color = getColorForImpactScore(impactScore)
        val icon = getIconForImpactScore(impactScore)

        return LineMarkerInfo(
            element,
            element.textRange,
            icon,
            { "Impact Score: ${(impactScore * 100).toInt()}%" },
            null,
            GutterIconRenderer.Alignment.RIGHT
        ) { _, _ ->
            // Show detailed history popup
            showHistoryPopup(element)
        }
    }

    private fun getColorForImpactScore(score: Float): Color {
        return when {
            score < 0.3f -> JBColor.GREEN
            score < 0.7f -> JBColor.YELLOW
            else -> JBColor.RED
        }
    }

    private fun getIconForImpactScore(score: Float): Icon {
        return when {
            score < 0.3f -> AllIcons.General.InspectionsOK
            score < 0.7f -> AllIcons.General.Warning
            else -> AllIcons.General.Error
        }
    }

    private fun showHistoryPopup(element: PsiElement) {
        // TODO: Implement history popup
    }

    companion object {
        private val HIGH_IMPACT_ATTRIBUTES = TextAttributes().apply {
            backgroundColor = JBColor(Color(255, 220, 220), Color(100, 40, 40))
            fontType = Font.BOLD
        }

        private val MEDIUM_IMPACT_ATTRIBUTES = TextAttributes().apply {
            backgroundColor = JBColor(Color(255, 255, 220), Color(100, 100, 40))
            fontType = Font.PLAIN
        }

        private val LOW_IMPACT_ATTRIBUTES = TextAttributes().apply {
            backgroundColor = JBColor(Color(220, 255, 220), Color(40, 100, 40))
            fontType = Font.PLAIN
        }
    }
} 