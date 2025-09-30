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

#[derive(Debug, Clone)]
pub struct App {
    #[allow(dead_code)]
    pub name: String,
    pub dir: PathBuf,
    pub dependencies: Vec<String>,
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

            apps.insert(
                app_name.clone(),
                App {
                    name: app_name,
                    dir: app_dir,
                    dependencies: config.app.dependencies,
                },
            );
        }
    }

    Ok(apps)
}

