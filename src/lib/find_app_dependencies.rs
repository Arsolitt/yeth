use crate::cfg::{App, Dependency};
use crate::error::YethError;
use std::collections::HashMap;

/// Find all dependencies for a specific app (including transitive dependencies)
pub fn find_app_dependencies(
    app_name: &str,
    apps: &HashMap<String, App>,
) -> Result<Vec<String>, YethError> {
    if !apps.contains_key(app_name) {
        return Err(YethError::AppNotFound(app_name.to_string()));
    }

    let mut visited = std::collections::HashSet::new();
    let mut result = Vec::new();
    let mut processing = std::collections::HashSet::new();
    
    fn dfs(
        current: &str,
        apps: &HashMap<String, App>,
        visited: &mut std::collections::HashSet<String>,
        processing: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>
    ) -> Result<(), YethError> {
        // Check if we're currently processing this node (cycle detection)
        if processing.contains(current) {
            return Ok(()); // Skip the rest of this branch to avoid infinite recursion
        }
        
        // If already visited, skip
        if visited.contains(current) {
            return Ok(());
        }
        
        // Mark as currently processing
        processing.insert(current.to_string());
        
        if let Some(app) = apps.get(current) {
            for dep in &app.dependencies {
                match dep {
                    Dependency::App(dep_name) => {
                        dfs(dep_name, apps, visited, processing, result)?;
                    }
                    Dependency::Path(_) => {
                        // Path dependencies don't need to be processed recursively
                    }
                }
            }
        }
        
        // Mark as visited and add to result
        processing.remove(current);
        visited.insert(current.to_string());
        result.push(current.to_string());
        Ok(())
    }
    
    dfs(app_name, apps, &mut visited, &mut processing, &mut result)?;
    
    // Result is already in correct order (dependencies first, then the app)
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::cfg::{App, Dependency};

    #[test]
    fn test_find_app_dependencies() {
        // Create a mock apps HashMap with dependencies
        let mut apps = HashMap::new();

        // App with no dependencies
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: PathBuf::from("/test/app1"),
                dependencies: vec![],
                exclude_patterns: vec![],
            },
        );

        // App that depends on app1
        apps.insert(
            "app2".to_string(),
            App {
                name: "app2".to_string(),
                dir: PathBuf::from("/test/app2"),
                dependencies: vec![Dependency::App("app1".to_string())],
                exclude_patterns: vec![],
            },
        );

        // App that depends on app2 (transitive dependency on app1)
        apps.insert(
            "app3".to_string(),
            App {
                name: "app3".to_string(),
                dir: PathBuf::from("/test/app3"),
                dependencies: vec![Dependency::App("app2".to_string())],
                exclude_patterns: vec![],
            },
        );

        // App with multiple dependencies
        apps.insert(
            "app4".to_string(),
            App {
                name: "app4".to_string(),
                dir: PathBuf::from("/test/app4"),
                dependencies: vec![
                    Dependency::App("app1".to_string()),
                    Dependency::App("app3".to_string()),
                ],
                exclude_patterns: vec![],
            },
        );

        // Test app with no dependencies
        let result = find_app_dependencies("app1", &apps).unwrap();
        assert_eq!(result, vec!["app1"]);

        // Test app with direct dependency
        let result = find_app_dependencies("app2", &apps).unwrap();
        assert_eq!(result, vec!["app1", "app2"]);

        // Test app with transitive dependency
        let result = find_app_dependencies("app3", &apps).unwrap();
        assert_eq!(result, vec!["app1", "app2", "app3"]);

        // Test app with multiple dependencies
        let result = find_app_dependencies("app4", &apps).unwrap();
        assert_eq!(result, vec!["app1", "app2", "app3", "app4"]);

        // Test non-existent app
        let result = find_app_dependencies("nonexistent", &apps);
        assert!(matches!(result, Err(YethError::AppNotFound(_))));
    }

    #[test]
    fn test_find_app_dependencies_with_path_dependencies() {
        let mut apps = HashMap::new();

        // App with path dependency
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: PathBuf::from("/test/app1"),
                dependencies: vec![Dependency::Path(PathBuf::from("/shared/lib"))],
                exclude_patterns: vec![],
            },
        );

        // App that depends on app1
        apps.insert(
            "app2".to_string(),
            App {
                name: "app2".to_string(),
                dir: PathBuf::from("/test/app2"),
                dependencies: vec![Dependency::App("app1".to_string())],
                exclude_patterns: vec![],
            },
        );

        // Test that path dependencies don't appear in the result
        let result = find_app_dependencies("app1", &apps).unwrap();
        assert_eq!(result, vec!["app1"]);

        // Test that app dependencies are still processed correctly
        let result = find_app_dependencies("app2", &apps).unwrap();
        assert_eq!(result, vec!["app1", "app2"]);
    }

    #[test]
    fn test_find_app_dependencies_with_circular_reference() {
        let mut apps = HashMap::new();

        // Create circular dependency: app1 -> app2 -> app1
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: PathBuf::from("/test/app1"),
                dependencies: vec![Dependency::App("app2".to_string())],
                exclude_patterns: vec![],
            },
        );

        apps.insert(
            "app2".to_string(),
            App {
                name: "app2".to_string(),
                dir: PathBuf::from("/test/app2"),
                dependencies: vec![Dependency::App("app1".to_string())],
                exclude_patterns: vec![],
            },
        );

        // The function should handle circular dependencies gracefully
        let result = find_app_dependencies("app1", &apps).unwrap();
        // Both apps should be in the result, but no infinite loop
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"app1".to_string()));
        assert!(result.contains(&"app2".to_string()));
    }
}
