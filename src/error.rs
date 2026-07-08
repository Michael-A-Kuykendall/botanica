use std::fmt;

/// Database error types for botanical operations
#[derive(Debug)]
pub enum DatabaseError {
    /// DuckDB database error
    DuckDbError(String),

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
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::DuckDbError(e) => write!(f, "Database error: {}", e),
            DatabaseError::MigrationError(msg) => write!(f, "Migration error: {}", msg),
            DatabaseError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            DatabaseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DatabaseError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DatabaseError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<duckdb::Error> for DatabaseError {
    fn from(error: duckdb::Error) -> Self {
        DatabaseError::DuckDbError(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for DatabaseError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        DatabaseError::DuckDbError(error.to_string())
    }
}

impl DatabaseError {
    pub fn migration<S: Into<String>>(msg: S) -> Self {
        DatabaseError::MigrationError(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ConfigError(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ValidationError(msg.into())
    }

    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        DatabaseError::NotFound(msg.into())
    }

    pub fn constraint<S: Into<String>>(msg: S) -> Self {
        DatabaseError::ConstraintViolation(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_database_error_display() {
        assert_eq!(
            DatabaseError::migration("Failed to run migration 001").to_string(),
            "Migration error: Failed to run migration 001"
        );
        assert_eq!(
            DatabaseError::config("Invalid database URL").to_string(),
            "Configuration error: Invalid database URL"
        );
        assert_eq!(
            DatabaseError::validation("Species name cannot be empty").to_string(),
            "Validation error: Species name cannot be empty"
        );
        assert_eq!(
            DatabaseError::not_found("Species with ID 12345 not found").to_string(),
            "Not found: Species with ID 12345 not found"
        );
        assert_eq!(
            DatabaseError::constraint("Foreign key constraint failed").to_string(),
            "Constraint violation: Foreign key constraint failed"
        );
    }

    #[test]
    fn test_database_error_convenience_constructors() {
        match DatabaseError::migration("test migration error") {
            DatabaseError::MigrationError(msg) => assert_eq!(msg, "test migration error"),
            _ => panic!("Expected MigrationError"),
        }
        match DatabaseError::config("test config error") {
            DatabaseError::ConfigError(msg) => assert_eq!(msg, "test config error"),
            _ => panic!("Expected ConfigError"),
        }
        match DatabaseError::validation("test validation error") {
            DatabaseError::ValidationError(msg) => assert_eq!(msg, "test validation error"),
            _ => panic!("Expected ValidationError"),
        }
        match DatabaseError::not_found("test not found error") {
            DatabaseError::NotFound(msg) => assert_eq!(msg, "test not found error"),
            _ => panic!("Expected NotFound"),
        }
        match DatabaseError::constraint("test constraint error") {
            DatabaseError::ConstraintViolation(msg) => assert_eq!(msg, "test constraint error"),
            _ => panic!("Expected ConstraintViolation"),
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = DatabaseError::validation("Test validation error");
        let _error_trait: &dyn Error = &error;
        assert!(error.source().is_none());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ValidationError"));
        assert!(debug_str.contains("Test validation error"));
    }

    #[test]
    fn test_error_consistency() {
        let errors = vec![
            (DatabaseError::migration("msg"), "Migration error: msg"),
            (DatabaseError::config("msg"), "Configuration error: msg"),
            (DatabaseError::validation("msg"), "Validation error: msg"),
            (DatabaseError::not_found("msg"), "Not found: msg"),
            (DatabaseError::constraint("msg"), "Constraint violation: msg"),
        ];
        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_string_conversion() {
        let error1 = DatabaseError::validation(String::from("string message"));
        let error2 = DatabaseError::validation("str message");
        assert_eq!(error1.to_string(), "Validation error: string message");
        assert_eq!(error2.to_string(), "Validation error: str message");
    }
}
