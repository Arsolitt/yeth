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

/// Паттерн исключения
#[derive(Debug, Clone)]
pub enum ExcludePattern {
    /// Простое имя (node_modules) - исключается везде где встретится
    Name(String),
    /// Абсолютный путь - исключается конкретный файл/директория
    AbsolutePath(PathBuf),
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
    pub exclude_patterns: Vec<ExcludePattern>,
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

            // Парсим паттерны исключений
            let exclude_patterns = config
                .app
                .exclude
                .iter()
                .map(|pattern| {
                    // Если содержит / или начинается с . - это путь
                    if pattern.contains('/') || pattern.starts_with('.') {
                        // Разрешаем относительно директории приложения
                        let abs_path = app_dir.join(pattern);
                        // Канонизируем путь (убираем .. и .)
                        let canonical = abs_path.canonicalize().unwrap_or(abs_path);
                        ExcludePattern::AbsolutePath(canonical)
                    } else {
                        // Простое имя
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

