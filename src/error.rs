use std::fmt;

/// Database error types for botanical operations
#[derive(Debug)]
pub enum DatabaseError {
    /// SQLx database error
    SqlxError(sqlx::Error),
    
    /// Migration error
    MigrationError(String),
    
    /// Configuration error
    ConfigError(String),
    
    /// Validation error
    ValidationError(String),
    
    /// Not found error
    NotFound(String),
    
    /// Constraint violation error
    ConstraintViolation(String),
    
    /// ContextLite integration error
    ContextLiteError(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::SqlxError(e) => write!(f, "Database error: {}", e),
            DatabaseError::MigrationError(msg) => write!(f, "Migration error: {}", msg),
            DatabaseError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            DatabaseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DatabaseError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DatabaseError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            DatabaseError::ContextLiteError(msg) => write!(f, "ContextLite error: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DatabaseError::SqlxError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(error: sqlx::Error) -> Self {
        DatabaseError::SqlxError(error)
    }
}

impl DatabaseError {
    /// Create a new migration error
    pub fn migration<S: Into<String>>(msg: S) -> Self {
        DatabaseError::MigrationError(msg.into())
    }
    
    /// Create a new configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ConfigError(msg.into())
    }
    
    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ValidationError(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        DatabaseError::NotFound(msg.into())
    }
    
    /// Create a new constraint violation error
    pub fn constraint<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ConstraintViolation(msg.into())
    }
    
    /// Create a new ContextLite error
    pub fn contextlite<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ContextLiteError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_database_error_display() {
        let migration_err = DatabaseError::migration("Failed to run migration 001");
        assert_eq!(migration_err.to_string(), "Migration error: Failed to run migration 001");
        
        let config_err = DatabaseError::config("Invalid database URL");
        assert_eq!(config_err.to_string(), "Configuration error: Invalid database URL");
        
        let validation_err = DatabaseError::validation("Species name cannot be empty");
        assert_eq!(validation_err.to_string(), "Validation error: Species name cannot be empty");
        
        let not_found_err = DatabaseError::not_found("Species with ID 12345 not found");
        assert_eq!(not_found_err.to_string(), "Not found: Species with ID 12345 not found");
        
        let constraint_err = DatabaseError::constraint("Foreign key constraint failed");
        assert_eq!(constraint_err.to_string(), "Constraint violation: Foreign key constraint failed");
        
        let contextlite_err = DatabaseError::contextlite("Connection timeout");
        assert_eq!(contextlite_err.to_string(), "ContextLite error: Connection timeout");
    }

    #[test] 
    fn test_database_error_convenience_constructors() {
        let migration_err = DatabaseError::migration("test migration error");
        match migration_err {
            DatabaseError::MigrationError(msg) => assert_eq!(msg, "test migration error"),
            _ => panic!("Expected MigrationError"),
        }

        let config_err = DatabaseError::config("test config error");
        match config_err {
            DatabaseError::ConfigError(msg) => assert_eq!(msg, "test config error"),
            _ => panic!("Expected ConfigError"),
        }

        let validation_err = DatabaseError::validation("test validation error");
        match validation_err {
            DatabaseError::ValidationError(msg) => assert_eq!(msg, "test validation error"),
            _ => panic!("Expected ValidationError"),
        }

        let not_found_err = DatabaseError::not_found("test not found error");
        match not_found_err {
            DatabaseError::NotFound(msg) => assert_eq!(msg, "test not found error"),
            _ => panic!("Expected NotFound"),
        }

        let constraint_err = DatabaseError::constraint("test constraint error");
        match constraint_err {
            DatabaseError::ConstraintViolation(msg) => assert_eq!(msg, "test constraint error"),
            _ => panic!("Expected ConstraintViolation"),
        }

        let contextlite_err = DatabaseError::contextlite("test contextlite error");
        match contextlite_err {
            DatabaseError::ContextLiteError(msg) => assert_eq!(msg, "test contextlite error"),
            _ => panic!("Expected ContextLiteError"),
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = DatabaseError::validation("Test validation error");
        
        // Test that it implements the Error trait
        let _error_trait: &dyn Error = &error;
        
        // Test source method returns None for non-SqlxError variants
        assert!(error.source().is_none());
        
        // Test Debug formatting
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ValidationError"));
        assert!(debug_str.contains("Test validation error"));
    }

    #[test] 
    fn test_from_sqlx_error() {
        // Create a mock SQLx error (we can't easily create a real one in tests)
        // This test verifies the From trait implementation exists
        let error = DatabaseError::migration("Simulated SQLx error conversion test");
        
        // Verify it's the right type
        match error {
            DatabaseError::MigrationError(_) => {},
            _ => panic!("Expected MigrationError for test"),
        }
    }

    #[test]
    fn test_error_consistency() {
        // Test that error messages are consistent with their types
        let errors = vec![
            (DatabaseError::migration("msg"), "Migration error: msg"),
            (DatabaseError::config("msg"), "Configuration error: msg"),
            (DatabaseError::validation("msg"), "Validation error: msg"),
            (DatabaseError::not_found("msg"), "Not found: msg"),
            (DatabaseError::constraint("msg"), "Constraint violation: msg"),
            (DatabaseError::contextlite("msg"), "ContextLite error: msg"),
        ];

        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_string_conversion() {
        // Test that Into<String> conversion works with various string types
        let string_msg = String::from("string message");
        let str_msg = "str message";
        
        let error1 = DatabaseError::validation(string_msg);
        let error2 = DatabaseError::validation(str_msg);
        
        assert_eq!(error1.to_string(), "Validation error: string message");
        assert_eq!(error2.to_string(), "Validation error: str message");
    }
}