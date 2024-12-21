use std::collections::HashMap;
use crate::{Category, Result};

pub struct ChangeClassifier {
    rules: HashMap<String, Category>,
}

impl ChangeClassifier {
    pub fn new() -> Result<Self> {
        let mut rules = HashMap::new();
        
        // Architecture patterns
        rules.insert("class".to_string(), Category::Architecture);
        rules.insert("struct".to_string(), Category::Architecture);
        rules.insert("interface".to_string(), Category::Architecture);
        rules.insert("trait".to_string(), Category::Architecture);
        
        // API patterns
        rules.insert("fn ".to_string(), Category::Api);
        rules.insert("pub ".to_string(), Category::Api);
        rules.insert("def ".to_string(), Category::Api);
        rules.insert("function".to_string(), Category::Api);
        
        // Logic patterns
        rules.insert("if ".to_string(), Category::Logic);
        rules.insert("match ".to_string(), Category::Logic);
        rules.insert("while ".to_string(), Category::Logic);
        rules.insert("for ".to_string(), Category::Logic);
        
        // Data patterns
        rules.insert("type ".to_string(), Category::Data);
        rules.insert("enum ".to_string(), Category::Data);
        rules.insert("struct ".to_string(), Category::Data);
        rules.insert("const ".to_string(), Category::Data);
        
        // Error handling patterns
        rules.insert("try".to_string(), Category::ErrorHandling);
        rules.insert("catch".to_string(), Category::ErrorHandling);
        rules.insert("throw".to_string(), Category::ErrorHandling);
        rules.insert("Result".to_string(), Category::ErrorHandling);
        
        // Logging patterns
        rules.insert("log".to_string(), Category::Logging);
        rules.insert("debug!".to_string(), Category::Logging);
        rules.insert("info!".to_string(), Category::Logging);
        rules.insert("warn!".to_string(), Category::Logging);
        
        // Documentation patterns
        rules.insert("///".to_string(), Category::Documentation);
        rules.insert("/**".to_string(), Category::Documentation);
        rules.insert("#[doc".to_string(), Category::Documentation);
        rules.insert("//!".to_string(), Category::Documentation);
        
        // Testing patterns
        rules.insert("#[test]".to_string(), Category::Testing);
        rules.insert("assert".to_string(), Category::Testing);
        rules.insert("expect".to_string(), Category::Testing);
        rules.insert("mock".to_string(), Category::Testing);
        
        // Performance patterns
        rules.insert("cache".to_string(), Category::Performance);
        rules.insert("optimize".to_string(), Category::Performance);
        rules.insert("performance".to_string(), Category::Performance);
        rules.insert("benchmark".to_string(), Category::Performance);
        
        // Security patterns
        rules.insert("encrypt".to_string(), Category::Security);
        rules.insert("decrypt".to_string(), Category::Security);
        rules.insert("auth".to_string(), Category::Security);
        rules.insert("password".to_string(), Category::Security);
        
        // Refactoring patterns
        rules.insert("refactor".to_string(), Category::Refactoring);
        rules.insert("rename".to_string(), Category::Refactoring);
        rules.insert("move".to_string(), Category::Refactoring);
        rules.insert("extract".to_string(), Category::Refactoring);
        
        // Dependencies patterns
        rules.insert("use ".to_string(), Category::Dependencies);
        rules.insert("import ".to_string(), Category::Dependencies);
        rules.insert("require".to_string(), Category::Dependencies);
        rules.insert("extern".to_string(), Category::Dependencies);
        
        // Configuration patterns
        rules.insert("config".to_string(), Category::Configuration);
        rules.insert("env".to_string(), Category::Configuration);
        rules.insert("setting".to_string(), Category::Configuration);
        rules.insert("flag".to_string(), Category::Configuration);
        
        // UI/UX patterns
        rules.insert("style".to_string(), Category::UiUx);
        rules.insert("css".to_string(), Category::UiUx);
        rules.insert("html".to_string(), Category::UiUx);
        rules.insert("layout".to_string(), Category::UiUx);
        
        // Accessibility patterns
        rules.insert("aria-".to_string(), Category::Accessibility);
        rules.insert("role=".to_string(), Category::Accessibility);
        rules.insert("alt=".to_string(), Category::Accessibility);
        rules.insert("a11y".to_string(), Category::Accessibility);

        Ok(Self { rules })
    }

    pub fn predict_categories(&self, diff: &str) -> Result<Vec<Category>> {
        let mut categories = std::collections::HashSet::new();
        
        // Process each line of the diff
        for line in diff.lines() {
            // Skip diff markers
            if line.starts_with("+++") || line.starts_with("---") || line.starts_with("@@") {
                continue;
            }
            
            // Remove diff prefix (+ or -)
            let line = line.trim_start_matches('+').trim_start_matches('-').trim();
            
            // Check each rule
            for (pattern, category) in &self.rules {
                if line.contains(pattern) {
                    categories.insert(category.clone());
                }
            }
        }
        
        Ok(categories.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_categories() {
        let classifier = ChangeClassifier::new().unwrap();
        
        // Test API and Logic changes
        let diff = "
            + pub fn new_function() {
            +     if true {
            +         // do something
            +     }
            + }
        ";
        let categories = classifier.predict_categories(diff).unwrap();
        assert!(categories.contains(&Category::Api));
        assert!(categories.contains(&Category::Logic));
        
        // Test Security changes
        let diff = "
            + fn encrypt_password(password: &str) {
            +     // encryption logic
            + }
        ";
        let categories = classifier.predict_categories(diff).unwrap();
        assert!(categories.contains(&Category::Security));
    }
} 