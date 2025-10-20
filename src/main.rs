mod cli;

use anyhow::Result;
use clap::Parser;
use yeth::{cfg::{App, Config, Dependency}, error::YethError, YethEngine};
use std::{collections::HashMap, time::Instant};

use cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse().validate()?;
    let start_time = Instant::now();

    let config = Config::builder().root(args.root).build()?;

    let engine = YethEngine::new(config);

    let apps = engine.discover_apps()?;

    if apps.is_empty() {
        return Err(YethError::NoApplicationsFound.into());
    }

    // If dependency graph requested
    if args.show_graph {
        print_dependency_graph(apps);
        return Ok(());
    }

    let ordered_apps = engine.topological_sort(&apps)?;
    let hashes = engine.calculate_hashes(ordered_apps, &apps)?;

    let format_hash = |hash: &str| -> String {
        if args.short_hash {
            hash.chars().take(args.short_hash_length).collect()
        } else {
            hash.to_string()
        }
    };

    // Save hashes to files if needed
    if args.write_versions {
        for (app_name, hash) in &hashes {
            let app = apps.get(app_name).unwrap();
            let version_file = app.dir.join("yeth.version");
            let formatted_hash = format_hash(hash);
            std::fs::write(&version_file, formatted_hash)?;
        }
    }

    // Output results
    if let Some(app_name) = &args.app {
        // Output for specific application
        if let Some(hash) = hashes.get(app_name) {
            let formatted_hash = format_hash(hash);
            if args.hash_only {
                println!("{}", formatted_hash);
            } else {
                println!("{} {}", formatted_hash, app_name);
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
            let formatted_hash = format_hash(hash);
            println!("{} {}", formatted_hash, app);
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

fn print_dependency_graph(apps: HashMap<String, App>) {
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
