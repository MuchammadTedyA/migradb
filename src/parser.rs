use crate::error::{MigrationError, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

const VERSION_FORMAT: &str = "%Y%m%d%H%M%S";
const VERSION_REGEX: &str = r"^(\d{14})_(.+)\.sql$";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationFile {
    pub version: String,
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
}

impl MigrationFile {
    pub fn new(version: String, name: String, path: PathBuf, content: String) -> Self {
        Self {
            version,
            name,
            path,
            content,
            applied: false,
            applied_at: None,
        }
    }

    pub fn parse_sql_statements(&self) -> Vec<String> {
        self.content
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .map(|s| s.replace("--", "").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Ord for MigrationFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialOrd for MigrationFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MigrationParser;

impl MigrationParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file(&self, path: &Path) -> Result<MigrationFile> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| MigrationError::Parse("Invalid filename".to_string()))?;

        let re = Regex::new(VERSION_REGEX).unwrap();
        let caps = re
            .captures(filename)
            .ok_or_else(|| MigrationError::InvalidVersion(filename.to_string()))?;

        let version = caps[1].to_string();
        let name = caps[2].to_string();
        let content = fs::read_to_string(path)?;

        Ok(MigrationFile::new(version, name, path.to_path_buf(), content))
    }

    pub fn parse_directory(&self, dir: &Path) -> Result<Vec<MigrationFile>> {
        if !dir.exists() {
            return Err(MigrationError::Config(format!(
                "Migrations directory does not exist: {}",
                dir.display()
            )));
        }

        let mut migrations = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("sql") {
                match self.parse_file(&path) {
                    Ok(migration) => migrations.push(migration),
                    Err(e) => log::warn!("Skipping file {}: {}", path.display(), e),
                }
            }
        }

        migrations.sort();
        Ok(migrations)
    }

    pub fn generate_version() -> String {
        Utc::now().format(VERSION_FORMAT).to_string()
    }

    pub fn create_migration_file(
        dir: &Path,
        name: &str,
        content: &str,
    ) -> Result<PathBuf> {
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }

        let version = Self::generate_version();
        let slug = name.to_lowercase().replace(' ', "_");
        let filename = format!("{}_{}.sql", version, slug);
        let path = dir.join(&filename);

        let template = format!(
            "-- Migration: {}\n-- Created at: {}\n\n{}\n",
            name,
            Utc::now().to_rfc3339(),
            content
        );

        fs::write(&path, template)?;
        Ok(path)
    }
}

impl Default for MigrationParser {
    fn default() -> Self {
        Self::new()
    }
}
