use std::path::Path;
use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
use plotters::prelude::*;
use crate::{Analysis, Change, Result, HistorianError};

pub struct Visualizer {
    output_dir: String,
}

impl Visualizer {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Result<Self> {
        let output_dir = output_dir
            .as_ref()
            .to_str()
            .ok_or_else(|| HistorianError::Analysis("Invalid output directory path".to_string()))?
            .to_string();

        Ok(Self { output_dir })
    }

    pub fn generate_timeline(&self, analysis: &Analysis) -> Result<String> {
        let mut graph = graph!(strict di id!("timeline"));
        
        let mut prev_node = None;
        for (i, change) in analysis.changes.iter().enumerate() {
            // Create node label
            let label = format!(
                "{}\n{}\n+{} -{}\n{}",
                change.timestamp.format("%Y-%m-%d"),
                change.author,
                change.diff.lines().filter(|l| l.starts_with('+')).count(),
                change.diff.lines().filter(|l| l.starts_with('-')).count(),
                change.categories.iter()
                    .take(3)
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            // Create node
            let node_id = format!("change_{}", i);
            let node_with_attrs = node!(node_id.clone();
                attr!("label") => label,
                attr!("shape") => "box",
                attr!("style") => "rounded"
            );
            graph.add_stmt(stmt!(node_with_attrs));

            // Add edge from previous node
            if let Some(prev) = prev_node {
                let edge = edge!(node_id!(prev) => node_id!(node_id.clone()));
                graph.add_stmt(stmt!(edge));
            }

            prev_node = Some(node_id);
        }

        // Generate DOT file
        let mut ctx = PrinterContext::default();
        let output_path = format!("{}/timeline.png", self.output_dir);

        // Generate PNG using Graphviz
        graphviz_rust::exec(
            graph,
            &mut ctx,
            vec![
                CommandArg::Format(Format::Png),
                CommandArg::Output(output_path.clone()),
            ],
        ).map_err(|e| HistorianError::Analysis(e.to_string()))?;

        Ok(output_path)
    }

    pub fn generate_category_distribution(&self, analysis: &Analysis) -> Result<String> {
        let output_path = format!("{}/category_distribution.png", self.output_dir);
        let output_path = output_path.clone();
        let root = BitMapBackend::new(&output_path, (800, 600))
            .into_drawing_area();
        
        root.fill(&WHITE)
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        let total_changes = analysis.metrics.total_changes as f64;
        let mut categories: Vec<_> = analysis.metrics.category_distribution
            .iter()
            .map(|(category, count)| {
                (format!("{:?}", category), *count as f64 / total_changes * 100.0)
            })
            .collect();
        categories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut chart = ChartBuilder::on(&root)
            .caption("Category Distribution", ("sans-serif", 30))
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(
                0..categories.len(),
                0f64..100f64,
            )
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&WHITE.mix(0.3))
            .y_desc("Percentage")
            .x_desc("Category")
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(BLUE.filled())
                .data(categories.iter().enumerate().map(|(i, (_, v))| (i, *v))),
        )
        .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        // Add category labels
        for (i, (category, _)) in categories.iter().enumerate() {
            chart.draw_series(std::iter::once(
                Text::new(
                    category.clone(),
                    (i, -5.0),
                    ("sans-serif", 12).into_font().transform(FontTransform::Rotate90),
                ),
            ))
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;
        }

        root.present()
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        Ok(output_path)
    }

    pub fn generate_impact_timeline(&self, changes: &[Change]) -> Result<String> {
        let output_path = format!("{}/impact_timeline.png", self.output_dir);
        let output_path = output_path.clone();
        let root = BitMapBackend::new(&output_path, (1024, 768))
            .into_drawing_area();
        
        root.fill(&WHITE)
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        let min_date = changes.last().unwrap().timestamp;
        let max_date = changes.first().unwrap().timestamp;

        let mut chart = ChartBuilder::on(&root)
            .caption("Impact Over Time", ("sans-serif", 30))
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(min_date..max_date, 0f64..10f64)
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        chart
            .configure_mesh()
            .x_labels(8)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|x| x.format("%Y-%m-%d").to_string())
            .draw()
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        // Draw impact scores
        chart.draw_series(LineSeries::new(
            changes.iter().map(|change| (change.timestamp, change.impact_score)),
            &BLUE,
        ))
        .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        // Add markers for significant changes
        chart.draw_series(
            changes
                .iter()
                .filter(|change| change.impact_score > 7.0)
                .map(|change| {
                    Circle::new(
                        (change.timestamp, change.impact_score),
                        3,
                        RED.filled(),
                    )
                }),
        )
        .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        root.present()
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::{TimeZone, Utc};
    use crate::Category;

    #[test]
    fn test_category_distribution() {
        let mut distribution = HashMap::new();
        distribution.insert(Category::Architecture, 10);
        distribution.insert(Category::Security, 5);

        let analysis = Analysis {
            changes: vec![],
            metrics: crate::Metrics {
                total_commits: 15,
                total_changes: 15,
                lines_added: 100,
                lines_removed: 50,
                category_distribution: distribution,
            },
            patterns: vec![],
        };

        let visualizer = Visualizer::new("/tmp").unwrap();
        let result = visualizer.generate_category_distribution(&analysis);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impact_timeline() {
        let changes = vec![
            Change {
                timestamp: Utc.ymd(2023, 1, 1).and_hms(0, 0, 0),
                author: "test".to_string(),
                commit_id: "1".to_string(),
                message: "test".to_string(),
                diff: "test".to_string(),
                categories: vec![Category::Architecture],
                impact_score: 8.0,
            },
            Change {
                timestamp: Utc.ymd(2023, 1, 2).and_hms(0, 0, 0),
                author: "test".to_string(),
                commit_id: "2".to_string(),
                message: "test".to_string(),
                diff: "test".to_string(),
                categories: vec![Category::Security],
                impact_score: 5.0,
            },
        ];

        let visualizer = Visualizer::new("/tmp").unwrap();
        let result = visualizer.generate_impact_timeline(&changes);
        assert!(result.is_ok());
    }
} 