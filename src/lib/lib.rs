pub mod cfg;
pub mod error;
mod find_app_dependencies;
mod topological_sort;

use cfg::{App, AppConfig, Dependency, ExcludePattern};
use error::YethError;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use std::path::Path;
use std::{collections::HashMap, fs, path::PathBuf};
use walkdir::WalkDir;

use crate::cfg::{Config, CONFIG_FILE};

pub struct YethEngine {
    config: Config,
}

impl YethEngine {
    pub fn new(config: Config) -> YethEngine {
        Self { config }
    }

    /// Find all dependencies for a specific app (including transitive dependencies)
    pub fn find_app_dependencies(&self, app_name: &str, apps: &HashMap<String, App>) -> Result<Vec<String>, YethError> {
      find_app_dependencies::find_app_dependencies(app_name, apps)
    }

    pub fn discover_apps(&self) -> Result<HashMap<String, App>, YethError> {
        WalkDir::new(&self.config.root)
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

    pub fn topological_sort(&self, apps: &HashMap<String, App>) -> Result<Vec<String>, YethError> {
      topological_sort::topological_sort(apps)
    }

    pub fn calculate_hashes(
        &self,
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
        &self,
        app_name: &str,
        apps: &HashMap<String, App>,
    ) -> Result<HashMap<String, String>, YethError> {
        // Find all dependencies for the specified app
        let dependency_order = self.find_app_dependencies(app_name, apps)?;
        
        // Calculate hashes only for the specified app and its dependencies
        self.calculate_hashes(dependency_order, apps)
    }
}

pub fn hash_directory(path: &PathBuf, exclude: &[ExcludePattern]) -> Result<String, YethError> {
    let mut hasher = Sha256::new();
    let mut files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if !e.file_type().is_file() {
                return false;
            }

            let entry_path = e.path();

            if entry_path
                .file_name()
                .is_some_and(|n| n == ".git" || n == ".DS_Store" || n == "yeth.version")
            {
                return false;
            }

            if should_exclude(entry_path, path, exclude) {
                return false;
            }

            true
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

pub fn compute_final_hash(own_hash: &str, dep_hashes: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(own_hash.as_bytes());
    for dep_hash in dep_hashes {
        hasher.update(dep_hash.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn hash_path(path: &Path, exclude: &[ExcludePattern]) -> Result<String, YethError> {
    if path.is_file() {
        hash_file(path)
    } else if path.is_dir() {
        hash_directory(&path.to_path_buf(), exclude)
    } else {
        Err(YethError::NorFileOrDirectory(path.to_path_buf()))
    }
}

fn should_exclude(path: &Path, base_dir: &Path, exclude_patterns: &[ExcludePattern]) -> bool {
    if exclude_patterns.is_empty() {
        return false;
    }

    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for pattern in exclude_patterns {
        match pattern {
            ExcludePattern::Name(name) => {
                let name_str = name.as_str();
                for component in path.components() {
                    if component.as_os_str().to_string_lossy() == name_str {
                        return true;
                    }
                }
            }
            ExcludePattern::AbsolutePath(abs_path) => {
                if canonical_path == *abs_path || canonical_path.starts_with(abs_path) {
                    return true;
                }
            }
        }
    }

    if let Ok(rel_path) = path.strip_prefix(base_dir) {
        let rel_path_str = rel_path.to_string_lossy();
        for pattern in exclude_patterns {
            if let ExcludePattern::Name(name) = pattern {
                let name_str = name.as_str();
                if rel_path_str.starts_with(name_str) || rel_path_str == name_str {
                    return true;
                }
            }
        }
    }

    false
}

// pub fn hash_file(path: &Path) -> Result<String, YethError> {
//     let mut hasher = Sha256::new();
//     let content = fs::read(path)?;
//     hasher.update(&content);
//     Ok(format!("{:x}", hasher.finalize()))
// }

pub fn hash_file(path: &Path) -> Result<String, YethError> {
  let mut hasher = Sha256::new();
  let file = fs::File::open(path)?;
  let mut reader = BufReader::new(file);
  
  let mut buffer = [0; 8192];
  loop {
      let bytes_read = reader.read(&mut buffer)?;
      if bytes_read == 0 {
          break;
      }
      hasher.update(&buffer[..bytes_read]);
  }
  
  Ok(format!("{:x}", hasher.finalize()))
}
