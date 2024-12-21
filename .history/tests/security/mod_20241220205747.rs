use std::path::Path;
use tokio::fs;
use serde_json::Value;
use reqwest::Client;
use std::time::Duration;
use futures::future::join_all;
use std::collections::HashMap;

pub struct SecurityTest {
    client: Client,
    base_url: String,
}

impl SecurityTest {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn test_api_authentication(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test unauthorized access
        let response = self.client
            .get(&format!("{}/api/metrics", self.base_url))
            .send()
            .await?;
        
        assert_eq!(response.status(), 401);

        // Test invalid API key
        let response = self.client
            .get(&format!("{}/api/metrics", self.base_url))
            .header("X-API-Key", "invalid-key")
            .send()
            .await?;
        
        assert_eq!(response.status(), 401);

        Ok(())
    }

    pub async fn test_input_validation(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test SQL injection prevention
        let malicious_input = "'; DROP TABLE users; --";
        let response = self.client
            .get(&format!("{}/api/history/file", self.base_url))
            .query(&[("path", malicious_input)])
            .send()
            .await?;
        
        assert_eq!(response.status(), 400);

        // Test path traversal prevention
        let malicious_path = "../../../etc/passwd";
        let response = self.client
            .get(&format!("{}/api/history/file", self.base_url))
            .query(&[("path", malicious_path)])
            .send()
            .await?;
        
        assert_eq!(response.status(), 400);

        Ok(())
    }

    pub async fn test_rate_limiting(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tasks = Vec::new();
        
        // Send 100 requests in parallel
        for _ in 0..100 {
            let client = self.client.clone();
            let url = format!("{}/api/metrics", self.base_url);
            
            tasks.push(tokio::spawn(async move {
                client.get(&url).send().await
            }));
        }

        let results = join_all(tasks).await;
        let mut responses = HashMap::new();

        for result in results {
            if let Ok(Ok(response)) = result {
                let status = response.status().as_u16();
                *responses.entry(status).or_insert(0) += 1;
            }
        }

        // Expect some requests to be rate limited
        assert!(responses.get(&429).unwrap_or(&0) > &0);

        Ok(())
    }

    pub async fn test_file_upload_security(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test file size limits
        let large_file = vec![0u8; 100 * 1024 * 1024]; // 100MB
        let response = self.client
            .post(&format!("{}/api/upload", self.base_url))
            .body(large_file)
            .send()
            .await?;
        
        assert_eq!(response.status(), 413);

        // Test file type validation
        let malicious_file = "malicious.exe";
        let response = self.client
            .post(&format!("{}/api/upload", self.base_url))
            .body(vec![0u8; 1024])
            .header("Content-Type", "application/x-msdownload")
            .send()
            .await?;
        
        assert_eq!(response.status(), 400);

        Ok(())
    }

    pub async fn test_data_encryption(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test TLS configuration
        let response = reqwest::get(&self.base_url.replace("http://", "https://"))
            .await?;
        
        assert!(response.status().is_success());

        // Verify secure headers
        let headers = response.headers();
        assert!(headers.contains_key("strict-transport-security"));
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("x-frame-options"));

        Ok(())
    }

    pub async fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running security tests...");

        println!("Testing API authentication...");
        self.test_api_authentication().await?;

        println!("Testing input validation...");
        self.test_input_validation().await?;

        println!("Testing rate limiting...");
        self.test_rate_limiting().await?;

        println!("Testing file upload security...");
        self.test_file_upload_security().await?;

        println!("Testing data encryption...");
        self.test_data_encryption().await?;

        println!("All security tests passed!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_suite() {
        let security_test = SecurityTest::new("http://localhost:8080");
        security_test.run_all_tests().await.unwrap();
    }
} 