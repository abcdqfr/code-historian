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
} 