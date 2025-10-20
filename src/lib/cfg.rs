use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

use crate::error::YethError;


pub const CONFIG_FILE: &str = "yeth.toml";

#[derive(Debug, Clone)]
pub struct Config {
    pub root: PathBuf,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    root: Option<PathBuf>,
}

impl ConfigBuilder {
    pub fn root(mut self, root: PathBuf) -> Self {
        self.root = Some(root);
        self
    }

    pub fn build(self) -> Result<Config, YethError> {
        Ok(Config {
            root: self.root.unwrap_or_else(|| PathBuf::from(".")),
        })
    }
}


#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub app: AppInfo,
}

#[derive(Deserialize, Debug)]
pub struct AppInfo {
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Exclusion pattern
#[derive(Debug, Clone)]
pub enum ExcludePattern {
    /// Simple name (node_modules) - excluded wherever it appears
    Name(String),
    /// Absolute path - excludes specific file/directory
    AbsolutePath(PathBuf),
}

/// Dependency type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    /// Dependency on another application
    App(String),
    /// Dependency on a file or directory
    Path(PathBuf),
}

impl Dependency {
    /// Parses dependency string and determines its type
    pub fn parse(dep_str: &str, app_dir: &Path) -> Self {
        // If string contains / or starts with . - it's a path
        if dep_str.contains('/') || dep_str.starts_with('.') {
            let path = app_dir.join(dep_str);
            Dependency::Path(path)
        } else {
            Dependency::App(dep_str.to_string())
        }
    }

    #[allow(dead_code)]
    pub fn as_app(&self) -> Option<&str> {
        match self {
            Dependency::App(name) => Some(name),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            Dependency::Path(path) => Some(path),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    #[allow(dead_code)]
    pub name: String,
    pub dir: PathBuf,
    pub dependencies: Vec<Dependency>,
    pub exclude_patterns: Vec<ExcludePattern>,
}
