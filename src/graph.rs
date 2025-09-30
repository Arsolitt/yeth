use anyhow::Result;
use std::collections::{HashMap, VecDeque};

use crate::config::App;

/// Строит граф зависимостей и возвращает топологически отсортированный список
pub fn topological_sort(apps: &HashMap<String, App>) -> Result<Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    // Строим граф и проверяем что все зависимости существуют
    for (app_name, app) in apps {
        for dep in &app.dependencies {
            if !apps.contains_key(dep) {
                return Err(anyhow::anyhow!(
                    "Зависимость '{}' для приложения '{}' не найдена",
                    dep,
                    app_name
                ));
            }
            graph
                .entry(dep.clone())
                .or_insert_with(Vec::new)
                .push(app_name.clone());
        }
        in_degree.insert(app_name.clone(), app.dependencies.len());
    }

    // Топологическая сортировка (Kahn's algorithm)
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

    // Проверка на циклические зависимости
    if topo_order.len() != apps.len() {
        return Err(anyhow::anyhow!("Обнаружена циклическая зависимость!"));
    }

    Ok(topo_order)
}

