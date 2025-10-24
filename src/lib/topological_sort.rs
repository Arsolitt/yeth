use crate::cfg::{App, Dependency};
use crate::error::YethError;
use std::collections::{HashMap, VecDeque};

/// Perform topological sort on applications based on their dependencies
pub fn topological_sort(apps: &HashMap<String, App>) -> Result<Vec<String>, YethError> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::with_capacity(apps.len());
    let mut in_degree: HashMap<String, usize> = HashMap::with_capacity(apps.len());

    for (app_name, app) in apps {
        let mut valid_app_deps = 0;

        for dep in &app.dependencies {
            match dep {
                Dependency::App(dep_name) => {
                    if !apps.contains_key(dep_name) {
                        return Err(YethError::DependencyNotFound(
                            dep_name.to_string(),
                            app_name.to_string(),
                        ));
                    }
                    graph
                        .entry(dep_name.clone())
                        .or_default()
                        .push(app_name.clone());
                    valid_app_deps += 1;
                }
                Dependency::Path(path) => {
                    if !path.exists() {
                        return Err(YethError::PathDependencyNotFound(
                            path.to_path_buf(),
                            app_name.to_string(),
                        ));
                    }
                }
            }
        }

        in_degree.insert(app_name.clone(), valid_app_deps);
    }

    let mut queue = VecDeque::with_capacity(in_degree.len());
    for (app, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(app.clone());
        }
    }

    let mut topo_order = Vec::with_capacity(in_degree.len());
    while let Some(app) = queue.pop_front() {
        topo_order.push(app.clone());
        if let Some(neighbors) = graph.get(&app) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if topo_order.len() != apps.len() {
        return Err(YethError::CircularDependency);
    }

    Ok(topo_order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::{App, Dependency};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_topological_sort() {
        // Create a test apps HashMap with dependencies
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

        // Test topological sort
        let result = topological_sort(&apps).unwrap();
        
        // Verify that dependencies come before dependents
        let app1_pos = result.iter().position(|x| x == "app1").unwrap();
        let app2_pos = result.iter().position(|x| x == "app2").unwrap();
        let app3_pos = result.iter().position(|x| x == "app3").unwrap();
        let app4_pos = result.iter().position(|x| x == "app4").unwrap();
        
        // app1 should come before app2 and app4
        assert!(app1_pos < app2_pos);
        assert!(app1_pos < app4_pos);
        
        // app2 should come before app3
        assert!(app2_pos < app3_pos);
        
        // app3 should come before app4
        assert!(app3_pos < app4_pos);
        
        // All apps should be in the result
        assert_eq!(result.len(), 4);
        assert!(result.contains(&"app1".to_string()));
        assert!(result.contains(&"app2".to_string()));
        assert!(result.contains(&"app3".to_string()));
        assert!(result.contains(&"app4".to_string()));
    }

    #[test]
    fn test_topological_sort_with_path_dependencies() {
        let mut apps = HashMap::new();
        
        // Create a temporary directory for the path dependency
        let temp_dir = std::env::temp_dir();
        let shared_lib = temp_dir.join("shared_lib");
        
        // App with path dependency to a valid path
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: PathBuf::from("/test/app1"),
                dependencies: vec![Dependency::Path(shared_lib.clone())],
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
        
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&shared_lib).unwrap();
        
        let result = topological_sort(&apps).unwrap();
        
        // app1 should come before app2
        let app1_pos = result.iter().position(|x| x == "app1").unwrap();
        let app2_pos = result.iter().position(|x| x == "app2").unwrap();
        assert!(app1_pos < app2_pos);
        
        // Clean up
        std::fs::remove_dir_all(&shared_lib).unwrap();
    }

    #[test]
    fn test_topological_sort_with_circular_dependency() {
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
        
        // Should return an error for circular dependencies
        let result = topological_sort(&apps);
        assert!(matches!(result, Err(YethError::CircularDependency)));
    }

    #[test]
    fn test_topological_sort_with_missing_dependency() {
        let mut apps = HashMap::new();
        
        // App with a dependency that doesn't exist
        apps.insert(
            "app1".to_string(),
            App {
                name: "app1".to_string(),
                dir: PathBuf::from("/test/app1"),
                dependencies: vec![Dependency::App("nonexistent".to_string())],
                exclude_patterns: vec![],
            },
        );
        
        // Should return an error for missing dependency
        let result = topological_sort(&apps);
        assert!(matches!(result, Err(YethError::DependencyNotFound(_, _))));
    }
}
