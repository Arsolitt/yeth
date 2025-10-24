use crate::cfg::{App, AppConfig, Config, Dependency, ExcludePattern, CONFIG_FILE};
use crate::error::YethError;
use std::{collections::HashMap, fs};
use walkdir::WalkDir;

/// Discover all applications in the configured root directory
pub fn discover_apps(config: &Config) -> Result<HashMap<String, App>, YethError> {
    WalkDir::new(&config.root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == CONFIG_FILE)
        .map(|entry| {
            let app_dir = entry
                .path()
                .parent()
                .ok_or_else(|| {
                    YethError::NoParentDir(entry.path().to_string_lossy().to_string())
                })?
                .to_path_buf();

            let app_name = app_dir
                .file_name()
                .ok_or_else(|| YethError::NoFileName(app_dir.to_string_lossy().to_string()))?
                .to_string_lossy()
                .into_owned();

            let app_config_content = fs::read_to_string(entry.path())?;
            let app_config: AppConfig = toml::from_str(&app_config_content)?;

            let dependencies = app_config
                .app
                .dependencies
                .iter()
                .map(|dep_string| Dependency::parse(dep_string, &app_dir))
                .collect::<Vec<Dependency>>();

            let exclude_patterns = app_config
                .app
                .exclude
                .iter()
                .map(|pattern| {
                    if pattern.contains("/") || pattern.starts_with(".") {
                        let absolute_path = app_dir.join(pattern);
                        ExcludePattern::AbsolutePath(
                            absolute_path.canonicalize().unwrap_or(absolute_path),
                        )
                    } else {
                        ExcludePattern::Name(pattern.clone())
                    }
                })
                .collect::<Vec<ExcludePattern>>();

            Ok((
                app_name.clone(),
                App {
                    name: app_name,
                    dir: app_dir,
                    dependencies,
                    exclude_patterns,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_apps() {
        // Create a temporary directory for our test
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create app1 directory and config
        let app1_dir = root.join("app1");
        fs::create_dir_all(&app1_dir).unwrap();
        let app1_config = app1_dir.join("yeth.toml");
        fs::write(&app1_config, r#"
[app]
dependencies = []
exclude = ["node_modules"]
"#).unwrap();

        // Create app2 directory with dependency on app1
        let app2_dir = root.join("app2");
        fs::create_dir_all(&app2_dir).unwrap();
        let app2_config = app2_dir.join("yeth.toml");
        fs::write(&app2_config, r#"
[app]
dependencies = ["app1"]
exclude = ["target", "*.log"]
"#).unwrap();

        // Create app3 directory with path dependency
        let app3_dir = root.join("app3");
        fs::create_dir_all(&app3_dir).unwrap();
        let app3_config = app3_dir.join("yeth.toml");
        fs::write(&app3_config, r#"
[app]
dependencies = ["../shared/lib"]
exclude = []
"#).unwrap();

        // Create a shared directory for path dependency
        let shared_dir = root.join("shared");
        fs::create_dir_all(&shared_dir.join("lib")).unwrap();

        // Create Config with our temporary directory as root
        let config = Config::builder().root(root.to_path_buf()).build().unwrap();

        // Test discover_apps
        let apps = discover_apps(&config).unwrap();

        // Verify we found all three apps
        assert_eq!(apps.len(), 3);
        assert!(apps.contains_key("app1"));
        assert!(apps.contains_key("app2"));
        assert!(apps.contains_key("app3"));

        // Verify app1 has no dependencies
        let app1 = apps.get("app1").unwrap();
        assert_eq!(app1.name, "app1");
        assert_eq!(app1.dir, app1_dir);
        assert_eq!(app1.dependencies.len(), 0);
        assert_eq!(app1.exclude_patterns.len(), 1);

        // Verify app2 has one dependency on app1
        let app2 = apps.get("app2").unwrap();
        assert_eq!(app2.name, "app2");
        assert_eq!(app2.dir, app2_dir);
        assert_eq!(app2.dependencies.len(), 1);
        match &app2.dependencies[0] {
            Dependency::App(name) => assert_eq!(name, "app1"),
            _ => panic!("Expected App dependency"),
        }
        assert_eq!(app2.exclude_patterns.len(), 2);

        // Verify app3 has one path dependency
        let app3 = apps.get("app3").unwrap();
        assert_eq!(app3.name, "app3");
        assert_eq!(app3.dir, app3_dir);
        assert_eq!(app3.dependencies.len(), 1);
        match &app3.dependencies[0] {
            Dependency::Path(path) => assert_eq!(path, &app3_dir.join("../shared/lib")),
            _ => panic!("Expected Path dependency"),
        }
        assert_eq!(app3.exclude_patterns.len(), 0);
    }

    #[test]
    fn test_discover_apps_empty_directory() {
        // Create a temporary directory with no apps
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create Config with our empty directory as root
        let config = Config::builder().root(root.to_path_buf()).build().unwrap();

        // Test discover_apps on empty directory
        let apps = discover_apps(&config).unwrap();

        // Verify we found no apps
        assert_eq!(apps.len(), 0);
    }

    #[test]
    fn test_discover_apps_with_invalid_config() {
        // Create a temporary directory for our test
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create app1 directory with invalid config
        let app1_dir = root.join("app1");
        fs::create_dir_all(&app1_dir).unwrap();
        let app1_config = app1_dir.join("yeth.toml");
        fs::write(&app1_config, "invalid toml content").unwrap();

        // Create Config with our temporary directory as root
        let config = Config::builder().root(root.to_path_buf()).build().unwrap();

        // Test discover_apps with invalid config
        let result = discover_apps(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), YethError::TomlParseError(_)));
    }
}
