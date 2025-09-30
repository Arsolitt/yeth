use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::ExcludePattern;

/// Проверяет, нужно ли исключить путь
fn should_exclude(path: &Path, base_dir: &Path, exclude_patterns: &[ExcludePattern]) -> bool {
    if exclude_patterns.is_empty() {
        return false;
    }

    // Канонизируем проверяемый путь
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for pattern in exclude_patterns {
        match pattern {
            ExcludePattern::Name(name) => {
                // Проверяем компоненты пути
                let name_str = name.as_str();
                for component in path.components() {
                    if component.as_os_str().to_string_lossy() == name_str {
                        return true;
                    }
                }
            }
            ExcludePattern::AbsolutePath(abs_path) => {
                // Проверяем точное совпадение или является ли path внутри abs_path
                if canonical_path == *abs_path || canonical_path.starts_with(abs_path) {
                    return true;
                }
            }
        }
    }

    // Дополнительно проверяем относительный путь для обратной совместимости
    if let Ok(rel_path) = path.strip_prefix(base_dir) {
        let rel_path_str = rel_path.to_string_lossy();
        for pattern in exclude_patterns {
            if let ExcludePattern::Name(name) = pattern {
                let name_str = name.as_str();
                // Префиксное совпадение для относительных путей
                if rel_path_str.starts_with(name_str) || rel_path_str == name_str {
                    return true;
                }
            }
        }
    }

    false
}

/// Вычисляет хэш пути (файла или директории)
pub fn hash_path(path: &Path, exclude: &[ExcludePattern]) -> Result<String> {
    if path.is_file() {
        hash_file(path)
    } else if path.is_dir() {
        hash_directory(&path.to_path_buf(), exclude)
    } else {
        Err(anyhow::anyhow!("Путь '{}' не является ни файлом, ни директорией", path.display()))
    }
}

/// Вычисляет хэш одного файла
pub fn hash_file(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let content = fs::read(path)?;
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Вычисляет хэш директории на основе содержимого всех файлов
pub fn hash_directory(path: &PathBuf, exclude: &[ExcludePattern]) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Пропускаем директории
            if !e.file_type().is_file() {
                return false;
            }

            let entry_path = e.path();

            // Игнорируем системные файлы
            let should_skip_system = entry_path.file_name().map_or(false, |n| {
                n == ".git" || n == ".DS_Store"
            });
            if should_skip_system {
                return false;
            }

            // Проверяем исключения
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

/// Вычисляет финальный хэш приложения с учётом зависимостей
pub fn compute_final_hash(own_hash: &str, dep_hashes: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(own_hash.as_bytes());
    for dep_hash in dep_hashes {
        hasher.update(dep_hash.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

