use std::path::Path;
use chrono::Utc;
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;
use crate::{Result, HistorianError, Analysis, Change, Pattern};

#[derive(Serialize)]
struct ReportContext {
    generated_at: String,
    total_commits: usize,
    total_changes: usize,
    lines_added: usize,
    lines_removed: usize,
    avg_impact: f64,
    high_impact_count: usize,
    most_impacted_files: Vec<String>,
    changes: Vec<Change>,
    patterns: Vec<Pattern>,
    category_labels: Vec<String>,
    category_data: Vec<usize>,
    timeline_labels: Vec<String>,
    timeline_data: Vec<usize>,
    impact_labels: Vec<String>,
    impact_data: Vec<f64>,
}

pub struct ReportGenerator {
    handlebars: Handlebars<'static>,
}

impl ReportGenerator {
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("report", include_str!("../templates/report.html"))?;
        
        Ok(Self { handlebars })
    }

    pub fn generate_html(&self, analysis: &Analysis, output_path: &Path) -> Result<()> {
        // Prepare context data
        let context = self.prepare_context(analysis)?;

        // Render template
        let html = self.handlebars.render("report", &context)
            .map_err(|e| HistorianError::Visualization(format!("Failed to render template: {}", e)))?;

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write HTML file
        std::fs::write(output_path, html)?;

        Ok(())
    }

    fn prepare_context(&self, analysis: &Analysis) -> Result<ReportContext> {
        // Calculate high impact changes
        let high_impact_threshold = 7.0;
        let high_impact_count = analysis.changes.iter()
            .filter(|c| c.impact_score >= high_impact_threshold)
            .count();

        // Calculate average impact
        let avg_impact = if !analysis.changes.is_empty() {
            analysis.changes.iter()
                .map(|c| c.impact_score)
                .sum::<f64>() / analysis.changes.len() as f64
        } else {
            0.0
        };

        // Get most impacted files
        let mut file_impacts: Vec<(String, f64)> = analysis.metrics.impact_distribution.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        file_impacts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let most_impacted_files = file_impacts.into_iter()
            .take(5)
            .map(|(file, _)| file)
            .collect();

        // Prepare category distribution data
        let mut category_data: Vec<(String, usize)> = analysis.metrics.category_distribution.iter()
            .map(|(k, v)| (format!("{:?}", k), *v))
            .collect();
        category_data.sort_by(|a, b| b.1.cmp(&a.1));

        let (category_labels, category_counts): (Vec<_>, Vec<_>) = category_data.into_iter().unzip();

        // Prepare timeline data
        let mut timeline_data: Vec<(String, usize)> = Vec::new();
        let mut changes_by_date: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for change in &analysis.changes {
            let date = change.timestamp.format("%Y-%m-%d").to_string();
            *changes_by_date.entry(date).or_insert(0) += 1;
        }

        let mut dates: Vec<_> = changes_by_date.keys().cloned().collect();
        dates.sort();

        for date in dates {
            timeline_data.push((
                date.clone(),
                *changes_by_date.get(&date).unwrap_or(&0),
            ));
        }

        let (timeline_labels, timeline_counts): (Vec<_>, Vec<_>) = timeline_data.into_iter().unzip();

        // Prepare impact distribution data
        let mut impact_data: Vec<(String, f64)> = Vec::new();
        let impact_ranges = vec![
            ("0-2", 0.0..2.0),
            ("2-4", 2.0..4.0),
            ("4-6", 4.0..6.0),
            ("6-8", 6.0..8.0),
            ("8-10", 8.0..10.0),
        ];

        for (label, range) in impact_ranges {
            let count = analysis.changes.iter()
                .filter(|c| range.contains(&c.impact_score))
                .count() as f64;
            impact_data.push((label.to_string(), count));
        }

        let (impact_labels, impact_counts): (Vec<_>, Vec<_>) = impact_data.into_iter().unzip();

        Ok(ReportContext {
            generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            total_commits: analysis.metrics.total_commits,
            total_changes: analysis.metrics.total_changes,
            lines_added: analysis.metrics.lines_added,
            lines_removed: analysis.metrics.lines_removed,
            avg_impact,
            high_impact_count,
            most_impacted_files,
            changes: analysis.changes.clone(),
            patterns: analysis.patterns.clone(),
            category_labels,
            category_data: category_counts,
            timeline_labels,
            timeline_data: timeline_counts,
            impact_labels,
            impact_data: impact_counts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Category;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("report.html");

        // Create test analysis data
        let mut category_distribution = HashMap::new();
        category_distribution.insert(Category::Performance, 5);
        category_distribution.insert(Category::Security, 3);

        let mut impact_distribution = HashMap::new();
        impact_distribution.insert("src/main.rs".to_string(), 8.5);
        impact_distribution.insert("src/lib.rs".to_string(), 6.2);

        let analysis = Analysis {
            changes: vec![
                Change {
                    commit_id: "test1".to_string(),
                    author: "Test Author".to_string(),
                    timestamp: Utc::now(),
                    message: "Test commit".to_string(),
                    file_path: "src/main.rs".into(),
                    diff: "test diff".to_string(),
                    categories: vec![Category::Performance],
                    impact_score: 8.5,
                    metrics: HashMap::new(),
                    annotations: vec![],
                },
            ],
            metrics: crate::analyzer::Metrics {
                total_commits: 1,
                total_changes: 1,
                lines_added: 10,
                lines_removed: 5,
                category_distribution,
                impact_distribution,
            },
            patterns: vec![
                Pattern {
                    name: "Test Pattern".to_string(),
                    description: "Test description".to_string(),
                    occurrences: 1,
                    impact: 8.5,
                    examples: vec!["Example 1".to_string()],
                },
            ],
            cache_info: None,
        };

        // Generate report
        let generator = ReportGenerator::new().unwrap();
        generator.generate_html(&analysis, &output_path).unwrap();

        // Verify file was created
        assert!(output_path.exists());

        // Verify file contains expected content
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Code Evolution Report"));
        assert!(content.contains("Test Pattern"));
        assert!(content.contains("Test Author"));
    }
} 