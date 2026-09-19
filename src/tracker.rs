use crate::error::{MigrationError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub version: String,
    pub applied_at: DateTime<Utc>,
}

impl MigrationRecord {
    pub fn new(version: String) -> Self {
        Self {
            version,
            applied_at: Utc::now(),
        }
    }
}

pub trait MigrationTracker: Send + Sync {
    fn initialize(&self) -> Result<()>;
    fn get_applied_versions(&self) -> Result<Vec<String>>;
    fn is_applied(&self, version: &str) -> Result<bool>;
    fn record_migration(&self, version: &str) -> Result<()>;
    fn remove_migration(&self, version: &str) -> Result<()>;
}

pub struct InMemoryTracker {
    records: std::sync::Mutex<Vec<MigrationRecord>>,
}

impl InMemoryTracker {
    pub fn new() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationTracker for InMemoryTracker {
    fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn get_applied_versions(&self) -> Result<Vec<String>> {
        let records = self.records.lock().unwrap();
        Ok(records.iter().map(|r| r.version.clone()).collect())
    }

    fn is_applied(&self, version: &str) -> Result<bool> {
        let records = self.records.lock().unwrap();
        Ok(records.iter().any(|r| r.version == version))
    }

    fn record_migration(&self, version: &str) -> Result<()> {
        let mut records = self.records.lock().unwrap();
        if !records.iter().any(|r| r.version == version) {
            records.push(MigrationRecord::new(version.to_string()));
        }
        Ok(())
    }

    fn remove_migration(&self, version: &str) -> Result<()> {
        let mut records = self.records.lock().unwrap();
        records.retain(|r| r.version != version);
        Ok(())
    }
}

pub struct SqlTracker {
    connection_string: String,
}

impl SqlTracker {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    pub fn initialize_sql() -> &'static str {
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#
    }

    pub fn select_versions_sql() -> &'static str {
        "SELECT version FROM schema_migrations ORDER BY version ASC;"
    }

    pub fn insert_version_sql() -> &'static str {
        "INSERT INTO schema_migrations (version) VALUES ($1) ON CONFLICT (version) DO NOTHING;"
    }

    pub fn delete_version_sql() -> &'static str {
        "DELETE FROM schema_migrations WHERE version = $1;"
    }
}
