use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum YethError {
    #[error("Application dependency '{0}' for '{1}' not found")]
    DependencyNotFound(String, String),

    #[error("Path dependency '{0}' for '{1}' not found")]
    PathDependencyNotFound(PathBuf, String),

    #[error("Path '{0}' is neither a file nor a directory")]
    NorFileOrDirectory(PathBuf),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Dependency not processed in correct order")]
    IncorrectOrder,

    #[error("Config file path has no parent directory: {0}")]
    NoParentDir(String),

    #[error("App directory path has no file name: {0}")]
    NoFileName(String),

    #[error("Failed to read config file: {0}")]
    ConfigReadError(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("No applications found")]
    NoApplicationsFound,

    #[error("Not implemented")]
    NotImplemented,
}
