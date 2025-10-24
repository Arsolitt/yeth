use crate::error::YethError;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;

/// Compute SHA256 hash for a file using buffered reading
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_hash_file() {
        // Create a temporary directory and file for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test_file.txt");
        
        // Write some content to the file
        let mut file = fs::File::create(&file_path).expect("Failed to create test file");
        file.write_all(b"Hello, World!").expect("Failed to write to test file");
        file.sync_all().expect("Failed to sync file");
        
        // Calculate the hash
        let hash_result = hash_file(&file_path);
        assert!(hash_result.is_ok(), "Failed to hash file: {:?}", hash_result.err());
        
        let hash = hash_result.unwrap();
        
        // Verify the hash is a valid SHA256 hash (64 hex characters)
        assert_eq!(hash.len(), 64, "Hash should be 64 characters long");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should contain only hex characters");
        
        // Test that the same file produces the same hash
        let hash_result2 = hash_file(&file_path);
        assert!(hash_result2.is_ok());
        let hash2 = hash_result2.unwrap();
        assert_eq!(hash, hash2, "Same file should produce the same hash");
        
        // Test that different content produces different hashes
        let mut file2 = fs::File::create(&file_path).expect("Failed to create test file");
        file2.write_all(b"Hello, Different World!").expect("Failed to write to test file");
        file2.sync_all().expect("Failed to sync file");
        
        let hash_result3 = hash_file(&file_path);
        assert!(hash_result3.is_ok());
        let hash3 = hash_result3.unwrap();
        assert_ne!(hash, hash3, "Different content should produce different hashes");
        
        // Test with a larger file to test the buffering
        let large_content = vec![0u8; 10000]; // 10KB of zeros
        let mut file3 = fs::File::create(&file_path).expect("Failed to create test file");
        file3.write_all(&large_content).expect("Failed to write to test file");
        file3.sync_all().expect("Failed to sync file");
        
        let hash_result4 = hash_file(&file_path);
        assert!(hash_result4.is_ok(), "Failed to hash large file: {:?}", hash_result4.err());
        let hash4 = hash_result4.unwrap();
        assert_eq!(hash4.len(), 64, "Hash of large file should be 64 characters long");
    }
}
