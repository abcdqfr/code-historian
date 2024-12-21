use std::path::PathBuf;
use assert_fs::TempDir;
use code_historian::{
    visualization::{
        TimelineGenerator,
        ChartRenderer,
        InteractiveVisualizer,
        TimelineOptions,
        ChartOptions,
        InteractiveOptions,
    },
    analysis::AnalysisResult,
    Change,
};
use chrono::{DateTime, Utc};

#[test]
fn test_timeline_generation() {
    let temp = TempDir::new().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir(&output_dir).unwrap();

    let changes = vec![
        Change {
            timestamp: Utc::now(),
            file: "src/main.rs".to_string(),
            lines_added: 10,
            lines_removed: 5,
            category: "Feature".to_string(),
            impact_score: 0.8,
        },
        Change {
            timestamp: Utc::now(),
            file: "src/lib.rs".to_string(),
            lines_added: 20,
            lines_removed: 15,
            category: "Refactor".to_string(),
            impact_score: 0.6,
        },
    ];

    let analysis = AnalysisResult {
        changes,
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    let options = TimelineOptions {
        width: 800,
        height: 400,
        show_categories: true,
        show_impact_scores: true,
    };

    let generator = TimelineGenerator::new(options);
    let result = generator.generate(&analysis, &output_dir.join("timeline.html"));
    assert!(result.is_ok());
    assert!(output_dir.join("timeline.html").exists());
}

#[test]
fn test_chart_rendering() {
    let temp = TempDir::new().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir(&output_dir).unwrap();

    let changes = vec![
        Change {
            timestamp: Utc::now(),
            file: "src/main.rs".to_string(),
            lines_added: 10,
            lines_removed: 5,
            category: "Feature".to_string(),
            impact_score: 0.8,
        },
        Change {
            timestamp: Utc::now(),
            file: "src/lib.rs".to_string(),
            lines_added: 20,
            lines_removed: 15,
            category: "Refactor".to_string(),
            impact_score: 0.6,
        },
    ];

    let analysis = AnalysisResult {
        changes,
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    let options = ChartOptions {
        width: 600,
        height: 400,
        chart_type: "pie".to_string(),
        show_legend: true,
    };

    let renderer = ChartRenderer::new(options);
    let result = renderer.render(&analysis, &output_dir.join("chart.html"));
    assert!(result.is_ok());
    assert!(output_dir.join("chart.html").exists());

    // Verify chart.js integration
    let html_content = std::fs::read_to_string(output_dir.join("chart.html")).unwrap();
    assert!(html_content.contains("chart.js"));
    assert!(html_content.contains("new Chart("));
}

#[test]
fn test_interactive_features() {
    let temp = TempDir::new().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir(&output_dir).unwrap();

    let changes = vec![
        Change {
            timestamp: Utc::now(),
            file: "src/main.rs".to_string(),
            lines_added: 10,
            lines_removed: 5,
            category: "Feature".to_string(),
            impact_score: 0.8,
        },
        Change {
            timestamp: Utc::now(),
            file: "src/lib.rs".to_string(),
            lines_added: 20,
            lines_removed: 15,
            category: "Refactor".to_string(),
            impact_score: 0.6,
        },
    ];

    let analysis = AnalysisResult {
        changes,
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    let options = InteractiveOptions {
        enable_zoom: true,
        enable_tooltips: true,
        enable_filtering: true,
        enable_sorting: true,
    };

    let visualizer = InteractiveVisualizer::new(options);
    let result = visualizer.create_visualization(&analysis, &output_dir.join("interactive.html"));
    assert!(result.is_ok());
    assert!(output_dir.join("interactive.html").exists());

    // Verify interactive features
    let html_content = std::fs::read_to_string(output_dir.join("interactive.html")).unwrap();
    assert!(html_content.contains("zoom"));
    assert!(html_content.contains("tooltip"));
    assert!(html_content.contains("filter"));
    assert!(html_content.contains("sort"));
}

#[test]
fn test_responsive_design() {
    let temp = TempDir::new().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir(&output_dir).unwrap();

    let changes = vec![
        Change {
            timestamp: Utc::now(),
            file: "src/main.rs".to_string(),
            lines_added: 10,
            lines_removed: 5,
            category: "Feature".to_string(),
            impact_score: 0.8,
        },
    ];

    let analysis = AnalysisResult {
        changes,
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    let options = InteractiveOptions {
        enable_zoom: true,
        enable_tooltips: true,
        enable_filtering: true,
        enable_sorting: true,
    };

    let visualizer = InteractiveVisualizer::new(options);
    visualizer.create_visualization(&analysis, &output_dir.join("responsive.html")).unwrap();

    // Verify responsive design elements
    let html_content = std::fs::read_to_string(output_dir.join("responsive.html")).unwrap();
    assert!(html_content.contains("@media"));
    assert!(html_content.contains("viewport"));
    assert!(html_content.contains("flex"));
}

#[test]
fn test_visualization_edge_cases() {
    let temp = TempDir::new().unwrap();
    let output_dir = temp.path().join("output");
    std::fs::create_dir(&output_dir).unwrap();

    // Test with empty changes
    let empty_analysis = AnalysisResult {
        changes: vec![],
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    let timeline_options = TimelineOptions {
        width: 800,
        height: 400,
        show_categories: true,
        show_impact_scores: true,
    };

    let generator = TimelineGenerator::new(timeline_options);
    assert!(generator.generate(&empty_analysis, &output_dir.join("empty_timeline.html")).is_ok());

    // Test with single change
    let single_change = AnalysisResult {
        changes: vec![Change {
            timestamp: Utc::now(),
            file: "src/main.rs".to_string(),
            lines_added: 1,
            lines_removed: 0,
            category: "Feature".to_string(),
            impact_score: 1.0,
        }],
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    assert!(generator.generate(&single_change, &output_dir.join("single_timeline.html")).is_ok());
} 