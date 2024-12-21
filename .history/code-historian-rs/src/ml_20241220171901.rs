use rust_bert::pipelines::sequence_classification::{
    SequenceClassificationModel, SequenceClassificationConfig,
};
use rust_bert::resources::RemoteResource;
use rust_bert::Config;
use tch::{Device, Tensor};
use crate::{Category, Result, HistorianError};

const MODEL_PATH: &str = "models/code-bert";
const VOCAB_PATH: &str = "models/code-bert/vocab.txt";
const CONFIG_PATH: &str = "models/code-bert/config.json";

pub struct ChangeClassifier {
    model: SequenceClassificationModel,
}

impl ChangeClassifier {
    pub fn new() -> Result<Self> {
        let config = SequenceClassificationConfig::new(
            15, // number of labels (categories)
            RemoteResource::from_pretrained(MODEL_PATH),
            RemoteResource::from_pretrained(VOCAB_PATH),
            RemoteResource::from_pretrained(CONFIG_PATH),
            None, // No merges file
            false, // Don't lower case
            Device::Cpu,
        );

        let model = SequenceClassificationModel::new(config)
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        Ok(Self { model })
    }

    pub fn predict_categories(&self, diff: &str) -> Result<Vec<Category>> {
        // Preprocess the diff text
        let processed_text = self.preprocess_diff(diff);

        // Get predictions from the model
        let outputs = self.model.predict(&[processed_text])
            .map_err(|e| HistorianError::Analysis(e.to_string()))?;

        // Convert predictions to categories
        let categories = self.convert_predictions_to_categories(&outputs[0]);

        Ok(categories)
    }

    fn preprocess_diff(&self, diff: &str) -> String {
        // Basic preprocessing
        diff.lines()
            .filter(|line| line.starts_with('+') || line.starts_with('-'))
            .map(|line| line[1..].trim())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn convert_predictions_to_categories(&self, predictions: &Tensor) -> Vec<Category> {
        let mut categories = Vec::new();
        let probabilities = predictions.softmax(-1, predictions.kind());
        
        // Get indices where probability is above threshold
        let threshold = 0.5;
        let indices = probabilities.gt(threshold).nonzero();
        
        for idx in indices.iter::<i64>().unwrap() {
            categories.push(match idx[0] {
                0 => Category::Architecture,
                1 => Category::Api,
                2 => Category::Logic,
                3 => Category::Data,
                4 => Category::ErrorHandling,
                5 => Category::Logging,
                6 => Category::Documentation,
                7 => Category::Testing,
                8 => Category::Performance,
                9 => Category::Security,
                10 => Category::Refactoring,
                11 => Category::Dependencies,
                12 => Category::Configuration,
                13 => Category::UiUx,
                14 => Category::Accessibility,
                _ => continue,
            });
        }

        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_diff() {
        let classifier = ChangeClassifier::new().unwrap();
        let diff = "
            + class NewFeature:
            +     def __init__(self):
            +         pass
            - class OldFeature:
            -     pass
        ";
        let processed = classifier.preprocess_diff(diff);
        assert!(processed.contains("class NewFeature"));
        assert!(processed.contains("class OldFeature"));
        assert!(!processed.contains("            +"));
    }

    // Note: This test requires the model files to be present
    #[test]
    #[ignore]
    fn test_predict_categories() {
        let classifier = ChangeClassifier::new().unwrap();
        let diff = "
            + def secure_function():
            +     encrypt_data()
            +     validate_input()
        ";
        let categories = classifier.predict_categories(diff).unwrap();
        assert!(categories.contains(&Category::Security));
    }
} 