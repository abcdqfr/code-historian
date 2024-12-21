use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HistorianError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Plugin load error: {0}")]
    PluginLoad(String),

    #[error("Plugin dependency error: {0}")]
    PluginDependency(String),

    #[error("Plugin validation error: {0}")]
    PluginValidation(String),

    #[error("Plugin execution error: {0}")]
    PluginExecution(String),

    #[error("Plugin configuration error: {0}")]
    PluginConfig(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Plugin version mismatch: {0}")]
    PluginVersion(String),

    #[error("Plugin manifest error: {0}")]
    PluginManifest(String),

    #[error("Directory error: path={path}, error={message}")]
    Directory {
        path: PathBuf,
        message: String,
    },

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Visualization error: {0}")]
    Visualization(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl HistorianError {
    /// Determines if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // IO errors that are recoverable
            HistorianError::Io(e) => matches!(
                e.kind(),
                std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::TimedOut
            ),

            // Git errors that are recoverable
            HistorianError::Git(e) => {
                let code = e.code();
                matches!(
                    code,
                    git2::ErrorCode::NotFound
                        | git2::ErrorCode::Exists
                        | git2::ErrorCode::Auth
                        | git2::ErrorCode::Network
                )
            }

            // Plugin errors that are recoverable
            HistorianError::Plugin(_)
            | HistorianError::PluginLoad(_)
            | HistorianError::PluginDependency(_)
            | HistorianError::PluginValidation(_)
            | HistorianError::PluginExecution(_)
            | HistorianError::PluginConfig(_)
            | HistorianError::PluginNotFound(_)
            | HistorianError::PluginVersion(_)
            | HistorianError::PluginManifest(_) => true,

            // Directory errors that are recoverable
            HistorianError::Directory { .. } => true,

            // Analysis errors that are recoverable
            HistorianError::Analysis(_) => true,

            // Visualization errors that are recoverable
            HistorianError::Visualization(_) => true,

            // Configuration errors that are recoverable
            HistorianError::Config(_) => true,

            // Non-recoverable errors
            HistorianError::InvalidArgument(_)
            | HistorianError::Unsupported(_)
            | HistorianError::Internal(_) => false,
        }
    }

    /// Attempts to recover from the error
    pub fn recover(&self) -> Result<(), Self> {
        if !self.is_recoverable() {
            return Err(self.clone());
        }

        match self {
            HistorianError::Io(e) => {
                match e.kind() {
                    std::io::ErrorKind::Interrupted => Ok(()), // Retry operation
                    std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        Ok(()) // Retry after delay
                    }
                    std::io::ErrorKind::TimedOut => {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        Ok(()) // Retry after longer delay
                    }
                    _ => Err(self.clone()),
                }
            }

            HistorianError::Git(e) => {
                match e.code() {
                    git2::ErrorCode::NotFound => Ok(()), // Skip missing item
                    git2::ErrorCode::Exists => Ok(()),   // Item already exists, continue
                    git2::ErrorCode::Auth | git2::ErrorCode::Network => {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        Ok(()) // Retry after delay
                    }
                    _ => Err(self.clone()),
                }
            }

            // Plugin errors - attempt to reload or skip
            HistorianError::Plugin(_)
            | HistorianError::PluginLoad(_)
            | HistorianError::PluginDependency(_)
            | HistorianError::PluginValidation(_)
            | HistorianError::PluginExecution(_)
            | HistorianError::PluginConfig(_)
            | HistorianError::PluginNotFound(_)
            | HistorianError::PluginVersion(_)
            | HistorianError::PluginManifest(_) => Ok(()), // Skip plugin and continue

            // Directory errors - attempt to create or use alternative
            HistorianError::Directory { .. } => Ok(()), // Use default directory

            // Analysis errors - skip problematic section
            HistorianError::Analysis(_) => Ok(()),

            // Visualization errors - use fallback visualization
            HistorianError::Visualization(_) => Ok(()),

            // Configuration errors - use default configuration
            HistorianError::Config(_) => Ok(()),

            // Non-recoverable errors
            _ => Err(self.clone()),
        }
    }
}

pub type Result<T> = std::result::Result<T, HistorianError>;

impl From<toml::de::Error> for HistorianError {
    fn from(err: toml::de::Error) -> Self {
        HistorianError::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for HistorianError {
    fn from(err: toml::ser::Error) -> Self {
        HistorianError::Config(err.to_string())
    }
}

impl From<serde_json::Error> for HistorianError {
    fn from(err: serde_json::Error) -> Self {
        HistorianError::Config(err.to_string())
    }
}

impl From<semver::Error> for HistorianError {
    fn from(err: semver::Error) -> Self {
        HistorianError::PluginVersion(err.to_string())
    }
}

impl From<libloading::Error> for HistorianError {
    fn from(err: libloading::Error) -> Self {
        HistorianError::PluginLoad(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let historian_error = HistorianError::from(io_error);
        assert!(matches!(historian_error, HistorianError::Io(_)));
    }

    #[test]
    fn test_plugin_errors() {
        let error = HistorianError::Plugin("test error".to_string());
        assert_eq!(error.to_string(), "Plugin error: test error");

        let error = HistorianError::PluginLoad("failed to load".to_string());
        assert_eq!(error.to_string(), "Plugin load error: failed to load");

        let error = HistorianError::PluginDependency("missing dep".to_string());
        assert_eq!(error.to_string(), "Plugin dependency error: missing dep");
    }

    #[test]
    fn test_directory_error() {
        let error = HistorianError::Directory {
            path: PathBuf::from("/test"),
            message: "access denied".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Directory error: path=/test, error=access denied"
        );
    }

    #[test]
    fn test_config_error() {
        let error = HistorianError::Config("invalid config".to_string());
        assert_eq!(error.to_string(), "Configuration error: invalid config");
    }

    #[test]
    fn test_error_recovery() {
        // Test recoverable errors
        let error = HistorianError::Plugin("test error".to_string());
        assert!(error.is_recoverable());
        assert!(error.recover().is_ok());

        let error = HistorianError::Config("invalid config".to_string());
        assert!(error.is_recoverable());
        assert!(error.recover().is_ok());

        // Test non-recoverable errors
        let error = HistorianError::InvalidArgument("invalid arg".to_string());
        assert!(!error.is_recoverable());
        assert!(error.recover().is_err());

        let error = HistorianError::Internal("internal error".to_string());
        assert!(!error.is_recoverable());
        assert!(error.recover().is_err());
    }

    #[test]
    fn test_io_error_recovery() {
        let error = HistorianError::Io(io::Error::new(
            io::ErrorKind::Interrupted,
            "interrupted",
        ));
        assert!(error.is_recoverable());
        assert!(error.recover().is_ok());

        let error = HistorianError::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "permission denied",
        ));
        assert!(!error.is_recoverable());
        assert!(error.recover().is_err());
    }
} 