use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Вычисляет хэш пути (файла или директории)
pub fn hash_path(path: &Path) -> Result<String> {
    if path.is_file() {
        hash_file(path)
    } else if path.is_dir() {
        hash_directory(&path.to_path_buf())
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
pub fn hash_directory(path: &PathBuf) -> Result<String> {
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

/// Вычисляет финальный хэш приложения с учётом зависимостей
pub fn compute_final_hash(own_hash: &str, dep_hashes: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(own_hash.as_bytes());
    for dep_hash in dep_hashes {
        hasher.update(dep_hash.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

