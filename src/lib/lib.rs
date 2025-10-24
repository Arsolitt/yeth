pub mod cfg;
pub mod error;
mod find_app_dependencies;
mod hash_file;
mod hash_directory;
mod topological_sort;
mod compute_final_hash;
mod discover_apps;
mod calculate_hashes;

use cfg::App;
use error::YethError;
use anyhow::Result;
use std::collections::HashMap;

use crate::cfg::Config;
use crate::discover_apps::discover_apps;
use crate::calculate_hashes::{calculate_hashes, calculate_hashes_for_app};

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
        calculate_hashes(ordered_apps, apps)
    }

    /// Calculate hashes for a specific app and its dependencies
    pub fn calculate_hashes_for_app(
        &self,
        app_name: &str,
        apps: &HashMap<String, App>,
    ) -> Result<HashMap<String, String>, YethError> {
        calculate_hashes_for_app(app_name, apps)
    }
}
