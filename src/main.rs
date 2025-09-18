use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use serde::Deserialize;
use toml;
use anyhow::Result;

const CONFIG_FILE: &str = "yeth.toml";

#[derive(Deserialize, Debug)]
struct AppConfig {
    app: AppInfo,
}

#[derive(Deserialize, Debug)]
struct AppInfo {
    dependencies: Vec<String>,
}

fn hash_directory(path: &PathBuf) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            // Игнорируем системные файлы
            let path = e.path();
            path.file_name().map_or(true, |n| {
                n != ".git" && n != ".idea" && n != ".DS_Store"
            })
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    files.sort();

    for file in files {
        let content = fs::read(&file)?;
        hasher.update(&content);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn main() -> Result<()> {
    let start_time = Instant::now();

    let root = PathBuf::from(".");
    let mut apps = HashMap::new();

    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == CONFIG_FILE {
            let app_dir = entry.path().parent().unwrap().to_path_buf();
            let app_name = app_dir.file_name().unwrap().to_string_lossy().into_owned();

            let config_content = fs::read_to_string(entry.path())?;
            let config: AppConfig = toml::from_str(&config_content)?;
            let dependencies = config.app.dependencies.clone();

            apps.insert(app_name.clone(), (app_dir.clone(), dependencies));
        }
    }

    let mut graph = HashMap::new();
    let mut in_degree = HashMap::new();

    for (app_name, (_, deps)) in &apps {
        for dep in deps {
            if !apps.contains_key(dep) {
                return Err(anyhow::anyhow!("Зависимость '{}' для приложения '{}' не найдена", dep, app_name));
            }
            graph.entry(dep.clone()).or_insert_with(Vec::new).push(app_name.clone());
        }
        in_degree.insert(app_name.clone(), deps.len());
    }

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


    if topo_order.len() != apps.len() {
        return Err(anyhow::anyhow!("Обнаружена циклическая зависимость!"));
    }

    let mut hashes = HashMap::new();
    for app_name in topo_order {
        let (app_dir, deps) = apps.get(&app_name).unwrap();
        let app_dir_pathbuf = app_dir.to_path_buf();
        let own_hash = hash_directory(&app_dir_pathbuf)?;
        
        let mut hasher = Sha256::new();
        hasher.update(own_hash.as_bytes());
        for dep in deps {
            let dep_hash: &String = hashes.get(dep).expect("Зависимость не обработана в правильном порядке");
            hasher.update(dep_hash.as_bytes());
        }
        let final_hash = format!("{:x}", hasher.finalize());
        hashes.insert(app_name.clone(), final_hash);
    }

    let mut sorted_apps: Vec<_> = hashes.keys().collect();
    sorted_apps.sort();
    for app in sorted_apps {
        let hash = hashes.get(app).unwrap();
        println!("{} {}", hash, app);
    }

    let elapsed_time = start_time.elapsed();    

    println!();
    println!("Время выполнения: {:.2?}", elapsed_time);
    println!("Обработано приложений: {}", hashes.len());

    Ok(())
}
