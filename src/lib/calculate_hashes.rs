use crate::cfg::{App, Dependency};
use crate::error::YethError;
use crate::compute_final_hash::compute_final_hash;
use crate::hash_directory::{hash_directory, hash_path};
use anyhow::Result;
use std::collections::HashMap;

/// Calculate hashes for a list of ordered applications
pub fn calculate_hashes(
    ordered_apps: Vec<String>,
    apps: &HashMap<String, App>,
) -> Result<HashMap<String, String>, YethError> {
    let mut hashes = HashMap::new();
    for app_name in ordered_apps {
        let app = apps.get(&app_name).unwrap();
        let own_hash = hash_directory(&app.dir, &app.exclude_patterns)?;

        let mut dep_hashes_owned: Vec<String> = Vec::new();

        for dep in &app.dependencies {
            match dep {
                Dependency::App(dep_name) => {
                    let dep_hash: &String =
                        hashes.get(dep_name).ok_or(YethError::IncorrectOrder)?;
                    dep_hashes_owned.push(dep_hash.clone());
                }
                Dependency::Path(path) => {
                    let path_hash = hash_path(path, &app.exclude_patterns)?;
                    dep_hashes_owned.push(path_hash);
                }
            }
        }

        let dep_hash_refs: Vec<&str> = dep_hashes_owned.iter().map(|s| s.as_str()).collect();
        let final_hash = compute_final_hash(&own_hash, &dep_hash_refs);

        hashes.insert(app_name.clone(), final_hash);
    }
    Ok(hashes)
}

/// Calculate hashes for a specific app and its dependencies
pub fn calculate_hashes_for_app(
    app_name: &str,
    apps: &HashMap<String, App>,
) -> Result<HashMap<String, String>, YethError> {
    // Find all dependencies for the specified app
    let dependency_order = crate::find_app_dependencies::find_app_dependencies(app_name, apps)?;
    
    // Calculate hashes only for the specified app and its dependencies
    calculate_hashes(dependency_order, apps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_calculate_hashes() {
        // Create a temporary directory for our test
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create app1 directory and files
        let app1_dir = root.join("app1");
        fs::create_dir_all(&app1_dir).unwrap();
        let app1_file1 = app1_dir.join("file1.txt");
        let app1_file2 = app1_dir.join("file2.txt");
        fs::write(&app1_file1, "App1 content 1").unwrap();
        fs::write(&app1_file2, "App1 content 2").unwrap();

        // Create app2 directory and files
        let app2_dir = root.join("app2");
        fs::create_dir_all(&app2_dir).unwrap();
        let app2_file1 = app2_dir.join("file1.txt");
        fs::write(&app2_file1, "App2 content").unwrap();

        // Create shared directory for path dependency
        let shared_dir = root.join("shared");
        fs::create_dir_all(&shared_dir).unwrap();
        let shared_file = shared_dir.join("lib.js");
        fs::write(&shared_file, "Shared library code").unwrap();

        // Create apps HashMap
        let mut apps = HashMap::new();

        // App1 with no dependencies
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: app1_dir.clone(),
                dependencies: vec![],
                exclude_patterns: vec![],
            },
        );

        // App2 with dependency on app1
        apps.insert(
            "app2".to_string(),
            App {
                name: "app2".to_string(),
                dir: app2_dir.clone(),
                dependencies: vec![Dependency::App("app1".to_string())],
                exclude_patterns: vec![],
            },
        );

        // App3 with path dependency
        let app3_dir = root.join("app3");
        fs::create_dir_all(&app3_dir).unwrap();
        let app3_file = app3_dir.join("file.txt");
        fs::write(&app3_file, "App3 content").unwrap();

        apps.insert(
            "app3".to_string(),
            App {
                name: "app3".to_string(),
                dir: app3_dir.clone(),
                dependencies: vec![Dependency::Path(shared_dir.clone())],
                exclude_patterns: vec![],
            },
        );

        // Test calculate_hashes with ordered apps
        let ordered_apps = vec!["app1".to_string(), "app2".to_string(), "app3".to_string()];
        let result = calculate_hashes(ordered_apps, &apps);

        assert!(result.is_ok(), "Failed to calculate hashes: {:?}", result.err());
        let hashes = result.unwrap();

        // Verify we have hashes for all apps
        assert_eq!(hashes.len(), 3);
        assert!(hashes.contains_key("app1"));
        assert!(hashes.contains_key("app2"));
        assert!(hashes.contains_key("app3"));

        // Verify hashes are valid SHA256 hashes (64 hex characters)
        for (app_name, hash) in &hashes {
            assert_eq!(hash.len(), 64, "Hash for {} should be 64 characters long", app_name);
            assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), 
                    "Hash for {} should contain only hex characters", app_name);
        }

        // Verify that app2's hash is different from app1's hash (due to dependency)
        let app1_hash = hashes.get("app1").unwrap();
        let app2_hash = hashes.get("app2").unwrap();
        assert_ne!(app1_hash, app2_hash, "App2 hash should be different from App1 hash");

        // Verify that app3's hash is different from app1's hash (due to path dependency)
        let app3_hash = hashes.get("app3").unwrap();
        assert_ne!(app1_hash, app3_hash, "App3 hash should be different from App1 hash");

        // Test that modifying a file changes the hash
        fs::write(&app1_file1, "Modified App1 content").unwrap();
        let ordered_apps = vec!["app1".to_string(), "app2".to_string()];
        let result = calculate_hashes(ordered_apps, &apps);
        assert!(result.is_ok());
        let new_hashes = result.unwrap();
        
        let new_app1_hash = new_hashes.get("app1").unwrap();
        let new_app2_hash = new_hashes.get("app2").unwrap();
        
        assert_ne!(app1_hash, new_app1_hash, "Modified file should change App1 hash");
        assert_ne!(app2_hash, new_app2_hash, "Modified dependency should change App2 hash");
    }

    #[test]
    fn test_calculate_hashes_with_incorrect_order() {
        // Create a temporary directory for our test
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create app1 directory and files
        let app1_dir = root.join("app1");
        fs::create_dir_all(&app1_dir).unwrap();
        let app1_file = app1_dir.join("file.txt");
        fs::write(&app1_file, "App1 content").unwrap();

        // Create app2 directory and files
        let app2_dir = root.join("app2");
        fs::create_dir_all(&app2_dir).unwrap();
        let app2_file = app2_dir.join("file.txt");
        fs::write(&app2_file, "App2 content").unwrap();

        // Create apps HashMap
        let mut apps = HashMap::new();

        // App1 with no dependencies
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: app1_dir,
                dependencies: vec![],
                exclude_patterns: vec![],
            },
        );

        // App2 with dependency on app1
        apps.insert(
            "app2".to_string(),
            App {
                name: "app2".to_string(),
                dir: app2_dir,
                dependencies: vec![Dependency::App("app1".to_string())],
                exclude_patterns: vec![],
            },
        );

        // Test calculate_hashes with incorrect order (app2 before app1)
        let ordered_apps = vec!["app2".to_string(), "app1".to_string()];
        let result = calculate_hashes(ordered_apps, &apps);

        // Should return an error due to incorrect order
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), YethError::IncorrectOrder));
    }
}
