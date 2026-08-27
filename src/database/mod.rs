use std::sync::Arc;
use tokio::sync::Mutex;
use crate::error::DatabaseError;

/// Configuration for the botanical database connection
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database file path or ":memory:" for in-memory
    pub url: String,

    /// Enable foreign key constraints (DuckDB enforces by default)
    pub foreign_keys: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "botanical.duckdb".to_string(),
            foreign_keys: true,
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration for in-memory database
    pub fn memory() -> Self {
        Self {
            url: ":memory:".to_string(),
            foreign_keys: true,
        }
    }

    /// Create a new database configuration for file-based database
    pub fn file<S: AsRef<str>>(path: S) -> Self {
        Self {
            url: path.as_ref().to_string(),
            foreign_keys: true,
        }
    }
}

/// Async wrapper around DuckDB connection
#[derive(Debug, Clone)]
pub struct BotanicalDatabase {
    conn: Arc<Mutex<duckdb::Connection>>,
}

impl BotanicalDatabase {
    /// Create a new database connection from configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        let url = config.url.clone();
        let conn = tokio::task::spawn_blocking(move || {
            duckdb::Connection::open(&url)
        })
        .await
        .map_err(|e| DatabaseError::config(format!("Failed to spawn blocking task: {}", e)))??;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create a new in-memory database for testing
    pub async fn memory() -> Result<Self, DatabaseError> {
        Self::new(DatabaseConfig::memory()).await
    }

    /// Run database migrations to set up tables
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        crate::migrations::run_migrations(&self).await
    }

    /// Check if the database connection is healthy
    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().await;
        conn.execute("SELECT 1", [])?;
        Ok(())
    }

    /// Get a reference to the underlying connection (for sync operations)
    pub async fn conn(&self) -> tokio::sync::MutexGuard<'_, duckdb::Connection> {
        self.conn.lock().await
    }



    /// Run a closure within a database transaction using SQL BEGIN/COMMIT/ROLLBACK
    /// The closure receives a reference to the DuckDB connection
    /// and should return Ok(()) on success or an error.
    pub async fn run_in_transaction<F>(&self, f: F) -> Result<(), DatabaseError>
    where
        F: FnOnce(&duckdb::Connection) -> Result<(), DatabaseError> + Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            conn.execute("BEGIN TRANSACTION", []).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
            match f(&conn) {
                Ok(()) => {
                    conn.execute("COMMIT", []).map_err(|e| DatabaseError::DuckDbError(e.to_string()))?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute("ROLLBACK", []);
                    Err(e)
                }
            }
        })
        .await
        .map_err(|e| DatabaseError::config(format!("Task failed: {}", e)))?
    }

    /// Close the database connection
    pub async fn close(&self) {
        // DuckDB connections close when dropped
        // The Arc<Mutex> will handle cleanup
    }

    /// Execute a SQL statement
    pub async fn execute(&self, sql: &str) -> Result<usize, DatabaseError> {
        let conn = self.conn.lock().await;
        let affected = conn.execute(sql, [])?;
        Ok(affected)
    }

    /// Execute a SQL statement with parameters
    pub async fn execute_named(
        &self,
        sql: &str,
        params: &[&dyn duckdb::ToSql],
    ) -> Result<usize, DatabaseError> {
        let conn = self.conn.lock().await;
        let affected = conn.execute(sql, params)?;
        Ok(affected)
    }
}
