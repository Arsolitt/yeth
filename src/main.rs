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

    // Находим все приложения
    let apps = discover_apps(&args.root)?;

    if apps.is_empty() {
        eprintln!("Не найдено приложений с yeth.toml");
        return Ok(());
    }

    // Если запрошен граф зависимостей
    if args.show_graph {
        print_dependency_graph(&apps);
        return Ok(());
    }

    // Топологическая сортировка
    let topo_order = topological_sort(&apps)?;

    // Вычисляем хэши
    let mut hashes = HashMap::new();
    for app_name in topo_order {
        let app = apps.get(&app_name).unwrap();
        let own_hash = hash_directory(&app.dir, &app.exclude_patterns)?;

        // Собираем хэши всех зависимостей (приложения + пути)
        let mut dep_hashes_owned: Vec<String> = Vec::new();
        
        for dep in &app.dependencies {
            match dep {
                Dependency::App(dep_name) => {
                    // Берём уже вычисленный хэш приложения
                    let dep_hash: &String = hashes
                        .get(dep_name)
                        .expect("Зависимость не обработана в правильном порядке");
                    dep_hashes_owned.push(dep_hash.clone());
                }
                Dependency::Path(path) => {
                    // Вычисляем хэш файла/директории на лету
                    // Используем exclude_patterns приложения - они уже разрешены в абсолютные пути
                    let path_hash = hash_path(path, &app.exclude_patterns)?;
                    dep_hashes_owned.push(path_hash);
                }
            }
        }

        let dep_hash_refs: Vec<&str> = dep_hashes_owned.iter().map(|s| s.as_str()).collect();
        let final_hash = compute_final_hash(&own_hash, &dep_hash_refs);

        hashes.insert(app_name.clone(), final_hash);
    }

    // Выводим результаты
    if let Some(app_name) = &args.app {
        // Вывод для конкретного приложения
        if let Some(hash) = hashes.get(app_name) {
            if args.hash_only {
                println!("{}", hash);
            } else {
                println!("{} {}", hash, app_name);
            }
        } else {
            eprintln!("Приложение '{}' не найдено", app_name);
            std::process::exit(1);
        }
    } else {
        // Вывод всех приложений
        let mut sorted_apps: Vec<_> = hashes.keys().collect();
        sorted_apps.sort();
        for app in sorted_apps {
            let hash = hashes.get(app).unwrap();
            println!("{} {}", hash, app);
        }
    }

    // Статистика
    if args.verbose {
        let elapsed_time = start_time.elapsed();
        println!();
        println!("Время выполнения: {:.2?}", elapsed_time);
        println!("Обработано приложений: {}", hashes.len());
    }

    Ok(())
}

fn print_dependency_graph(apps: &HashMap<String, config::App>) {
    println!("Граф зависимостей:\n");
    let mut sorted_apps: Vec<_> = apps.keys().collect();
    sorted_apps.sort();

    for app_name in sorted_apps {
        let app = apps.get(app_name).unwrap();
        println!("{}", app_name);
        if app.dependencies.is_empty() {
            println!("  └─ (нет зависимостей)");
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
