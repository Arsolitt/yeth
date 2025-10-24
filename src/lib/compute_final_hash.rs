use sha2::{Digest, Sha256};

/// Compute the final hash by combining the app's own hash with its dependencies' hashes
pub fn compute_final_hash(own_hash: &str, dep_hashes: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(own_hash.as_bytes());
    for dep_hash in dep_hashes {
        hasher.update(dep_hash.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_final_hash() {
        // Test with empty dependencies
        let own_hash = "a1b2c3d4e5f6";
        let dep_hashes: Vec<&str> = vec![];
        let result = compute_final_hash(own_hash, &dep_hashes);
        
        // The result should be different from the own hash when no dependencies
        assert_ne!(result, own_hash);
        assert_eq!(result.len(), 64); // SHA256 hex length
        
        // Test with single dependency
        let dep_hash1 = "f6e5d4c3b2a1";
        let dep_hashes: Vec<&str> = vec![dep_hash1];
        let result = compute_final_hash(own_hash, &dep_hashes);
        
        // The result should be different from both inputs
        assert_ne!(result, own_hash);
        assert_ne!(result, dep_hash1);
        assert_eq!(result.len(), 64);
        
        // Test with multiple dependencies
        let dep_hash2 = "z9y8x7w6v5u4";
        let dep_hashes: Vec<&str> = vec![dep_hash1, dep_hash2];
        let result = compute_final_hash(own_hash, &dep_hashes);
        
        // The result should be different from all inputs
        assert_ne!(result, own_hash);
        assert_ne!(result, dep_hash1);
        assert_ne!(result, dep_hash2);
        assert_eq!(result.len(), 64);
        
        // Test that the same inputs always produce the same output
        let result1 = compute_final_hash(own_hash, &dep_hashes);
        let result2 = compute_final_hash(own_hash, &dep_hashes);
        assert_eq!(result1, result2);
        
        // Test that different dependency order produces different results
        let dep_hashes_reordered: Vec<&str> = vec![dep_hash2, dep_hash1];
        let result_reordered = compute_final_hash(own_hash, &dep_hashes_reordered);
        assert_ne!(result, result_reordered);
    }
}
