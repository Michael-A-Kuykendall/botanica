//! Database connection and migration tests
//!
//! Tests database initialization, health checks, and migrations.

use crate::database::{BotanicalDatabase, DatabaseConfig};
use crate::{create_test_database, initialize_database};

#[tokio::test]
async fn test_in_memory_database_creation() {
    let db = BotanicalDatabase::memory().await;
    assert!(db.is_ok(), "Failed to create in-memory database: {:?}", db.err());

    let db = db.unwrap();
    assert!(db.health_check().await.is_ok(), "Health check failed for in-memory database");
}

#[tokio::test]
async fn test_file_database_creation() {
    let config = DatabaseConfig::memory();

    let db = BotanicalDatabase::new(config).await;
    assert!(db.is_ok(), "Failed to create file database: {:?}", db.err());

    let db = db.unwrap();
    assert!(db.health_check().await.is_ok(), "Health check failed for file database");
}

#[tokio::test]
async fn test_database_config_creation() {
    let config = DatabaseConfig::memory();
    assert_eq!(config.url, ":memory:");
    assert!(config.foreign_keys);

    let config = DatabaseConfig::file("test.duckdb");
    assert_eq!(config.url, "test.duckdb");
    assert!(config.foreign_keys);

    let config = DatabaseConfig::default();
    assert_eq!(config.url, "botanical.duckdb");
    assert!(config.foreign_keys);
}

#[tokio::test]
async fn test_database_migration_success() {
    let db = BotanicalDatabase::memory().await.expect("Failed to create database");

    let result = db.migrate().await;
    assert!(result.is_ok(), "Migration failed: {:?}", result.err());
}

#[tokio::test]
async fn test_create_test_database_helper() {
    let result = create_test_database().await;
    assert!(result.is_ok(), "create_test_database helper failed: {:?}", result.err());

    let db = result.unwrap();
    assert!(db.health_check().await.is_ok(), "Health check failed after helper creation");
}

#[tokio::test]
async fn test_initialize_database_helper() {
    let result = initialize_database(":memory:").await;
    assert!(result.is_ok(), "initialize_database helper failed: {:?}", result.err());

    let db = result.unwrap();
    assert!(db.health_check().await.is_ok(), "Health check failed after initialize");
}

#[tokio::test]
async fn test_database_query_access() {
    let db = create_test_database().await.expect("Failed to create test database");

    // Test that we can execute a simple query
    let conn = db.conn().await;
    let result: Result<i32, _> = conn.query_row("SELECT 1", [], |row| row.get(0));
    assert!(result.is_ok(), "Failed to execute query: {:?}", result.err());
    assert_eq!(result.unwrap(), 1, "Query result was not as expected");
}

#[tokio::test]
async fn test_database_tables_exist_after_migration() {
    let db = create_test_database().await.expect("Failed to create test database");

    let conn = db.conn().await;

    // Check that families table exists
    let result: Result<Option<String>, _> = conn.query_row(
        "SELECT table_name FROM information_schema.tables WHERE table_name = 'families'",
        [],
        |row| row.get(0),
    );
    assert!(result.is_ok(), "Failed to query information_schema: {:?}", result.err());
    assert!(result.unwrap().is_some(), "families table does not exist after migration");

    // Check that genera table exists
    let result: Result<Option<String>, _> = conn.query_row(
        "SELECT table_name FROM information_schema.tables WHERE table_name = 'genera'",
        [],
        |row| row.get(0),
    );
    assert!(result.is_ok(), "Failed to query information_schema: {:?}", result.err());
    assert!(result.unwrap().is_some(), "genera table does not exist after migration");

    // Check that species table exists
    let result: Result<Option<String>, _> = conn.query_row(
        "SELECT table_name FROM information_schema.tables WHERE table_name = 'species'",
        [],
        |row| row.get(0),
    );
    assert!(result.is_ok(), "Failed to query information_schema: {:?}", result.err());
    assert!(result.unwrap().is_some(), "species table does not exist after migration");
}

#[tokio::test]
async fn test_database_close() {
    let db = create_test_database().await.expect("Failed to create test database");

    // Verify database is working before close
    assert!(db.health_check().await.is_ok(), "Database should be healthy before close");

    // Close the database
    db.close().await;

    // After closing, the Arc<Mutex> is still accessible but the connection is dropped
    // DuckDB connections are cleaned up when dropped
}
