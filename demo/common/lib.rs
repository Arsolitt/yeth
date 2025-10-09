// Common Rust utilities
use std::collections::HashMap;

pub fn build_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("version".to_string(), "1.0.0".to_string());
    config.insert("env".to_string(), "production".to_string());
    config
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid"));
    }
}

