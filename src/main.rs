mod cli;
mod config;
mod graph;
mod hash;

use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::time::Instant;

use cli::Cli;
use config::{discover_apps, Dependency};
use graph::topological_sort;
use hash::{compute_final_hash, hash_directory, hash_path};

fn main() -> Result<()> {
    let args = Cli::parse();
    let start_time = Instant::now();

    // Find all applications
    let apps = discover_apps(&args.root)?;

    if apps.is_empty() {
        eprintln!("No applications with yeth.toml found");
        return Ok(());
    }

    // If dependency graph requested
    if args.show_graph {
        print_dependency_graph(&apps);
        return Ok(());
    }

    // Topological sort
    let topo_order = topological_sort(&apps)?;

    // Calculate hashes
    let mut hashes = HashMap::new();
    for app_name in topo_order {
        let app = apps.get(&app_name).unwrap();
        let own_hash = hash_directory(&app.dir, &app.exclude_patterns)?;

        // Collect hashes of all dependencies (apps + paths)
        let mut dep_hashes_owned: Vec<String> = Vec::new();
        
        for dep in &app.dependencies {
            match dep {
                Dependency::App(dep_name) => {
                    // Get already calculated application hash
                    let dep_hash: &String = hashes
                        .get(dep_name)
                        .expect("Dependency not processed in correct order");
                    dep_hashes_owned.push(dep_hash.clone());
                }
                Dependency::Path(path) => {
                    // Calculate file/directory hash on the fly
                    // Use application's exclude_patterns - they are already resolved to absolute paths
                    let path_hash = hash_path(path, &app.exclude_patterns)?;
                    dep_hashes_owned.push(path_hash);
                }
            }
        }

        let dep_hash_refs: Vec<&str> = dep_hashes_owned.iter().map(|s| s.as_str()).collect();
        let final_hash = match args.short_hash {
            true => compute_final_hash(&own_hash, &dep_hash_refs).chars().take(args.short_hash_length).collect(),
            false => compute_final_hash(&own_hash, &dep_hash_refs),
        };

        hashes.insert(app_name.clone(), final_hash);
    }

    // Save hashes to files if needed
    if args.write_versions {
        for (app_name, hash) in &hashes {
            let app = apps.get(app_name).unwrap();
            let version_file = app.dir.join("yeth.version");
            std::fs::write(&version_file, hash)?;
        }
    }

    // Output results
    if let Some(app_name) = &args.app {
        // Output for specific application
        if let Some(hash) = hashes.get(app_name) {
            if args.hash_only {
                println!("{}", hash);
            } else {
                println!("{} {}", hash, app_name);
            }
        } else {
            eprintln!("Application '{}' not found", app_name);
            std::process::exit(1);
        }
    } else {
        // Output all applications
        let mut sorted_apps: Vec<_> = hashes.keys().collect();
        sorted_apps.sort();
        for app in sorted_apps {
            let hash = hashes.get(app).unwrap();
            println!("{} {}", hash, app);
        }
    }

    // Statistics
    if args.verbose {
        let elapsed_time = start_time.elapsed();
        println!();
        println!("Execution time: {:.2?}", elapsed_time);
        println!("Applications processed: {}", hashes.len());
    }

    Ok(())
}

fn print_dependency_graph(apps: &HashMap<String, config::App>) {
    println!("Dependency graph:\n");
    let mut sorted_apps: Vec<_> = apps.keys().collect();
    sorted_apps.sort();

    for app_name in sorted_apps {
        let app = apps.get(app_name).unwrap();
        println!("{}", app_name);
        if app.dependencies.is_empty() {
            println!("  └─ (no dependencies)");
        } else {
            for (i, dep) in app.dependencies.iter().enumerate() {
                let prefix = if i == app.dependencies.len() - 1 {
                    "└─"
                } else {
                    "├─"
                };
                
                match dep {
                    Dependency::App(dep_name) => {
                        println!("  {} {} (app)", prefix, dep_name);
                    }
                    Dependency::Path(path) => {
                        let path_str = path.display();
                        let kind = if path.is_file() { "file" } else { "dir" };
                        println!("  {} {} ({})", prefix, path_str, kind);
                    }
                }
            }
        }
        println!();
    }
}
