pub mod cfg;
pub mod error;
mod find_app_dependencies;
mod hash_file;
mod hash_directory;
mod topological_sort;
mod compute_final_hash;
mod discover_apps;

use cfg::{App, Dependency};
use error::YethError;
use anyhow::Result;
use std::collections::HashMap;

use crate::cfg::Config;
use compute_final_hash::compute_final_hash;
use hash_directory::{hash_directory, hash_path};
use crate::discover_apps::discover_apps;

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
        discover_apps(&self.config)
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
