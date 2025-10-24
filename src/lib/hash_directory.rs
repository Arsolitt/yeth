use crate::cfg::ExcludePattern;
use crate::error::YethError;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Compute SHA256 hash for a directory by hashing all files in it
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

/// Compute hash for a path (file or directory)
pub fn hash_path(path: &Path, exclude: &[ExcludePattern]) -> Result<String, YethError> {
    if path.is_file() {
        crate::hash_file::hash_file(path)
    } else if path.is_dir() {
        hash_directory(&path.to_path_buf(), exclude)
    } else {
        Err(YethError::NorFileOrDirectory(path.to_path_buf()))
    }
}

/// Check if a path should be excluded based on exclusion patterns
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_hash_directory() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let dir_path = temp_dir.path();
        
        // Create some test files
        let file1_path = dir_path.join("file1.txt");
        let file2_path = dir_path.join("file2.txt");
        let sub_dir = dir_path.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");
        let file3_path = sub_dir.join("file3.txt");
        
        // Write content to files
        fs::write(&file1_path, "Hello, World!").expect("Failed to write file1");
        fs::write(&file2_path, "Another file").expect("Failed to write file2");
        fs::write(&file3_path, "Nested file").expect("Failed to write file3");
        
        // Hash the directory
        let hash_result = hash_directory(&dir_path.to_path_buf(), &[]);
        assert!(hash_result.is_ok(), "Failed to hash directory: {:?}", hash_result.err());
        
        let hash = hash_result.unwrap();
        
        // Verify the hash is a valid SHA256 hash (64 hex characters)
        assert_eq!(hash.len(), 64, "Hash should be 64 characters long");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should contain only hex characters");
        
        // Test that the same directory produces the same hash
        let hash_result2 = hash_directory(&dir_path.to_path_buf(), &[]);
        assert!(hash_result2.is_ok());
        let hash2 = hash_result2.unwrap();
        assert_eq!(hash, hash2, "Same directory should produce the same hash");
        
        // Test that modifying a file changes the hash
        fs::write(&file1_path, "Modified content").expect("Failed to modify file1");
        let hash_result3 = hash_directory(&dir_path.to_path_buf(), &[]);
        assert!(hash_result3.is_ok());
        let hash3 = hash_result3.unwrap();
        assert_ne!(hash, hash3, "Modified directory should produce different hash");
    }

    #[test]
    fn test_hash_directory_with_exclusions() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let dir_path = temp_dir.path();
        
        // Create some test files
        let file1_path = dir_path.join("file1.txt");
        let file2_path = dir_path.join("file2.txt");
        let node_modules = dir_path.join("node_modules");
        fs::create_dir(&node_modules).expect("Failed to create node_modules directory");
        let lib_file = node_modules.join("lib.js");
        
        // Write content to files
        fs::write(&file1_path, "Hello, World!").expect("Failed to write file1");
        fs::write(&file2_path, "Another file").expect("Failed to write file2");
        fs::write(&lib_file, "Library code").expect("Failed to write lib file");
        
        // Hash without exclusions
        let hash_all = hash_directory(&dir_path.to_path_buf(), &[]).unwrap();
        
        // Hash with name exclusion
        let exclude_patterns = vec![ExcludePattern::Name("node_modules".to_string())];
        let hash_excluded = hash_directory(&dir_path.to_path_buf(), &exclude_patterns).unwrap();
        
        // Hashes should be different when excluding files
        assert_ne!(hash_all, hash_excluded, "Hashes should be different when excluding files");
        
        // Test with absolute path exclusion
        let abs_exclude_patterns = vec![ExcludePattern::AbsolutePath(node_modules.clone())];
        let hash_abs_excluded = hash_directory(&dir_path.to_path_buf(), &abs_exclude_patterns).unwrap();
        
        // Should be the same as name exclusion
        assert_eq!(hash_excluded, hash_abs_excluded, "Name and absolute path exclusion should produce same result");
    }

    #[test]
    fn test_hash_directory_ignores_special_files() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let dir_path = temp_dir.path();
        
        // Create some test files including special ones
        let file1_path = dir_path.join("file1.txt");
        let git_file = dir_path.join(".git");  // This is a file named .git, not a directory
        let ds_store = dir_path.join(".DS_Store");
        let version_file = dir_path.join("yeth.version");
        
        // Write content to files
        fs::write(&file1_path, "Hello, World!").expect("Failed to write file1");
        fs::write(&git_file, "Git file").expect("Failed to write git file");
        fs::write(&ds_store, "DS Store").expect("Failed to write DS Store");
        fs::write(&version_file, "1.0.0").expect("Failed to write version file");
        
        // Hash the directory
        let hash_result = hash_directory(&dir_path.to_path_buf(), &[]);
        assert!(hash_result.is_ok());
        
        // Now delete the special files and hash again
        fs::remove_file(&git_file).expect("Failed to remove git file");
        fs::remove_file(&ds_store).expect("Failed to remove DS Store");
        fs::remove_file(&version_file).expect("Failed to remove version file");
        
        let hash_result2 = hash_directory(&dir_path.to_path_buf(), &[]);
        assert!(hash_result2.is_ok());
        
        // Hashes should be the same since special files are ignored
        assert_eq!(hash_result.unwrap(), hash_result2.unwrap(), 
                  "Hashes should be the same since special files are ignored");
    }
}
