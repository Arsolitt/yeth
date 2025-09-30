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
}

/// Тип зависимости
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    /// Зависимость от другого приложения
    App(String),
    /// Зависимость от файла или директории
    Path(PathBuf),
}

impl Dependency {
    /// Парсит строку зависимости и определяет её тип
    pub fn parse(dep_str: &str, app_dir: &PathBuf) -> Self {
        // Если строка содержит / или начинается с . - это путь
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
}

/// Сканирует директорию в поисках приложений с yeth.toml
pub fn discover_apps(root: &PathBuf) -> Result<HashMap<String, App>> {
    let mut apps = HashMap::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == CONFIG_FILE {
            let app_dir = entry.path().parent().unwrap().to_path_buf();
            let app_name = app_dir.file_name().unwrap().to_string_lossy().into_owned();

            let config_content = fs::read_to_string(entry.path())?;
            let config: AppConfig = toml::from_str(&config_content)?;

            // Парсим зависимости
            let dependencies = config
                .app
                .dependencies
                .iter()
                .map(|dep_str| Dependency::parse(dep_str, &app_dir))
                .collect();

            apps.insert(
                app_name.clone(),
                App {
                    name: app_name,
                    dir: app_dir,
                    dependencies,
                },
            );
        }
    }

    Ok(apps)
}

