mod cli;

use anyhow::Result;
use clap::Parser;
use yeth::{cfg::{App, Config, Dependency}, error::YethError, YethEngine};
use std::{collections::HashMap, time::Instant};

use cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse().validate()?;
    
    // Check if benchmarking mode is enabled
    if let Some(iterations) = args.bench {
        return run_benchmark(args, iterations);
    }
    
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
    let hashes = if let Some(app_name) = &args.app {
        engine.calculate_hashes_for_app(app_name, &apps)?
    } else {
        engine.calculate_hashes(ordered_apps, &apps)?
    };

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

fn run_benchmark(mut args: Cli, iterations: usize) -> Result<()> {
    // Disable verbose for individual runs, we'll show our own stats
    let original_verbose = args.verbose;
    args.verbose = false;
    
    println!("Running benchmark with {} iterations...", iterations);
    println!();
    
    let mut total_times = Vec::with_capacity(iterations);
    let mut apps_count = 0;
    
    for i in 1..=iterations {
        let start_time = Instant::now();
        
        // Run the processing
        let config = Config::builder().root(args.root.clone()).build()?;
        let engine = YethEngine::new(config);
        let apps = engine.discover_apps()?;
        
        if apps.is_empty() {
            return Err(YethError::NoApplicationsFound.into());
        }
        
        // Store apps count from first iteration
        if i == 1 {
            apps_count = apps.len();
        }
        
        let ordered_apps = engine.topological_sort(&apps)?;
        let _hashes = if let Some(app_name) = &args.app {
            engine.calculate_hashes_for_app(app_name, &apps)?
        } else {
            engine.calculate_hashes(ordered_apps, &apps)?
        };
        
        let elapsed = start_time.elapsed();
        total_times.push(elapsed);
        
        if original_verbose {
            println!("Iteration {}: {:.2?}", i, elapsed);
        }
    }
    
    // Calculate statistics
    let total_duration: std::time::Duration = total_times.iter().sum();
    let average_time = total_duration / iterations as u32;
    let min_time = total_times.iter().min().unwrap();
    let max_time = total_times.iter().max().unwrap();
    
    // Calculate median
    let mut sorted_times = total_times.clone();
    sorted_times.sort();
    let median_time = if iterations % 2 == 0 {
        // Even number of iterations - average of two middle values
        let mid1 = sorted_times[iterations / 2 - 1];
        let mid2 = sorted_times[iterations / 2];
        (mid1 + mid2) / 2
    } else {
        // Odd number of iterations - middle value
        sorted_times[iterations / 2]
    };
    
    // Calculate standard deviation
    let variance: f64 = total_times.iter()
        .map(|&x| {
            let diff = x.as_secs_f64() - average_time.as_secs_f64();
            diff * diff
        })
        .sum::<f64>() / iterations as f64;
    let std_dev = variance.sqrt();
    
    println!("Benchmark results:");
    println!("  Iterations: {}", iterations);
    println!("  Applications processed: {}", apps_count);
    println!("  Average time: {:.2?}", average_time);
    println!("  Median time: {:.2?}", median_time);
    println!("  Min time: {:.2?}", min_time);
    println!("  Max time: {:.2?}", max_time);
    println!("  Standard deviation: {:.2?}", std::time::Duration::from_secs_f64(std_dev));
    println!("  Total time: {:.2?}", total_duration);
    
    Ok(())
}
