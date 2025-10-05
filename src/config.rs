use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub const CONFIG_FILE: &str = "yeth.toml";

#[derive(Deserialize, Debug)]
struct AppConfig {
    app: AppInfo,
}

#[derive(Deserialize, Debug)]
struct AppInfo {
    dependencies: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
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
    pub fn parse(dep_str: &str, app_dir: &PathBuf) -> Self {
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

/// Scans directory for applications with yeth.toml
pub fn discover_apps(root: &PathBuf) -> Result<HashMap<String, App>> {
    let mut apps = HashMap::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == CONFIG_FILE {
            let app_dir = entry.path().parent().unwrap().to_path_buf();
            let app_name = app_dir.file_name().unwrap().to_string_lossy().into_owned();

            let config_content = fs::read_to_string(entry.path())?;
            let config: AppConfig = toml::from_str(&config_content)?;

            // Parse dependencies
            let dependencies = config
                .app
                .dependencies
                .iter()
                .map(|dep_str| Dependency::parse(dep_str, &app_dir))
                .collect();

            // Parse exclusion patterns
            let exclude_patterns = config
                .app
                .exclude
                .iter()
                .map(|pattern| {
                    // If contains / or starts with . - it's a path
                    if pattern.contains('/') || pattern.starts_with('.') {
                        // Resolve relative to application directory
                        let abs_path = app_dir.join(pattern);
                        // Canonicalize path (remove .. and .)
                        let canonical = abs_path.canonicalize().unwrap_or(abs_path);
                        ExcludePattern::AbsolutePath(canonical)
                    } else {
                        // Simple name
                        ExcludePattern::Name(pattern.clone())
                    }
                })
                .collect();

            apps.insert(
                app_name.clone(),
                App {
                    name: app_name,
                    dir: app_dir,
                    dependencies,
                    exclude_patterns,
                },
            );
        }
    }

    Ok(apps)
}

