use anyhow::Result;
use std::collections::{HashMap, VecDeque};

use crate::config::{App, Dependency};

/// Builds dependency graph and returns topologically sorted list
pub fn topological_sort(apps: &HashMap<String, App>) -> Result<Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    // Build graph and check that all dependencies exist
    for (app_name, app) in apps {
        let mut valid_app_deps = 0;
        
        for dep in &app.dependencies {
            match dep {
                Dependency::App(dep_name) => {
                    // Check that application dependency exists
                    if !apps.contains_key(dep_name) {
                        return Err(anyhow::anyhow!(
                            "Application dependency '{}' for '{}' not found",
                            dep_name,
                            app_name
                        ));
                    }
                    graph
                        .entry(dep_name.clone())
                        .or_insert_with(Vec::new)
                        .push(app_name.clone());
                    valid_app_deps += 1;
                }
                Dependency::Path(path) => {
                    // Check that path exists
                    if !path.exists() {
                        return Err(anyhow::anyhow!(
                            "Path dependency '{}' for '{}' not found",
                            path.display(),
                            app_name
                        ));
                    }
                }
            }
        }
        
        // In-degree only counts dependencies on other applications
        in_degree.insert(app_name.clone(), valid_app_deps);
    }

    // Topological sort (Kahn's algorithm)
    let mut queue = VecDeque::new();
    for (app, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(app.clone());
        }
    }

    let mut topo_order = Vec::new();
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

    // Check for circular dependencies
    if topo_order.len() != apps.len() {
        return Err(anyhow::anyhow!("Circular dependency detected!"));
    }

    Ok(topo_order)
}

