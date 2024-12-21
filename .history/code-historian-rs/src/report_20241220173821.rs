use std::path::Path;
use crate::{Analysis, Category, Result, HistorianError};

pub struct ReportGenerator {
    output_dir: std::path::PathBuf,
}

impl ReportGenerator {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    pub fn generate_report(&self, analysis: &Analysis) -> Result<()> {
        let report_path = self.output_dir.join("REPORT.md");
        let mut report = String::new();

        // Add header
        report.push_str("# Code Analysis Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Local::now()));

        // Add summary
        self.add_summary(&mut report, analysis);

        // Add category distribution
        self.add_category_distribution(&mut report, analysis);

        // Add patterns section
        self.add_patterns(&mut report, analysis);

        // Add visualizations section
        self.add_visualizations(&mut report);

        // Write report to file
        std::fs::write(&report_path, report)
            .map_err(|e| HistorianError::Io(e))?;

        Ok(())
    }

    fn add_summary(&self, report: &mut String, analysis: &Analysis) {
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- Total commits analyzed: {}\n", analysis.metrics.total_commits));
        report.push_str(&format!("- Total changes: {}\n", analysis.metrics.total_changes));
        report.push_str(&format!("- Lines added: {}\n", analysis.metrics.lines_added));
        report.push_str(&format!("- Lines removed: {}\n", analysis.metrics.lines_removed));
        report.push_str("\n");
    }

    fn add_category_distribution(&self, report: &mut String, analysis: &Analysis) {
        report.push_str("## Category Distribution\n\n");
        report.push_str("| Category | Count | Percentage |\n");
        report.push_str("|----------|--------|------------|\n");

        let total = analysis.metrics.total_changes as f64;
        for (category, count) in &analysis.metrics.category_distribution {
            let percentage = (*count as f64 / total) * 100.0;
            report.push_str(&format!(
                "| {:?} | {} | {:.1}% |\n",
                category, count, percentage
            ));
        }
        report.push_str("\n");
    }

    fn add_patterns(&self, report: &mut String, analysis: &Analysis) {
        report.push_str("## Detected Patterns\n\n");
        for pattern in &analysis.patterns {
            report.push_str(&format!("### {}\n", pattern.name));
            report.push_str(&format!("- Description: {}\n", pattern.description));
            report.push_str(&format!("- Occurrences: {}\n", pattern.occurrences));
            report.push_str(&format!("- Impact Score: {:.2}\n", pattern.impact));
            report.push_str("\n");
        }
    }

    fn add_visualizations(&self, report: &mut String) {
        report.push_str("## Visualizations\n\n");
        
        let files = [
            ("Timeline", "timeline.png"),
            ("Category Distribution", "category_distribution.png"),
            ("Impact Timeline", "impact_timeline.png"),
        ];

        for (name, file) in files {
            if self.output_dir.join(file).exists() {
                report.push_str(&format!("- [{}]({})\n", name, file));
            }
        }
        report.push_str("\n");
    }

    pub fn generate_json_report(&self, analysis: &Analysis) -> Result<()> {
        let report_path = self.output_dir.join("report.json");
        let json = serde_json::to_string_pretty(analysis)
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;
        
        std::fs::write(&report_path, json)
            .map_err(|e| HistorianError::Io(e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_analysis() -> Analysis {
        let mut category_distribution = HashMap::new();
        category_distribution.insert(Category::Architecture, 10);
        category_distribution.insert(Category::Security, 5);

        Analysis {
            changes: vec![],
            metrics: crate::Metrics {
                total_commits: 15,
                total_changes: 15,
                lines_added: 100,
                lines_removed: 50,
                category_distribution,
            },
            patterns: vec![
                crate::Pattern {
                    name: "Test Pattern".to_string(),
                    description: "A test pattern".to_string(),
                    occurrences: 5,
                    impact: 0.8,
                },
            ],
        }
    }

    #[test]
    fn test_generate_report() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ReportGenerator::new(temp_dir.path());
        let analysis = create_test_analysis();

        let result = generator.generate_report(&analysis);
        assert!(result.is_ok());

        let report_path = temp_dir.path().join("REPORT.md");
        assert!(report_path.exists());

        let content = std::fs::read_to_string(report_path).unwrap();
        assert!(content.contains("Code Analysis Report"));
        assert!(content.contains("Total commits analyzed: 15"));
    }

    #[test]
    fn test_generate_json_report() {
        let temp_dir = TempDir::new().unwrap();
        let generator = ReportGenerator::new(temp_dir.path());
        let analysis = create_test_analysis();

        let result = generator.generate_json_report(&analysis);
        assert!(result.is_ok());

        let report_path = temp_dir.path().join("report.json");
        assert!(report_path.exists());

        let content = std::fs::read_to_string(report_path).unwrap();
        assert!(content.contains("total_commits"));
        assert!(content.contains("category_distribution"));
    }
} 