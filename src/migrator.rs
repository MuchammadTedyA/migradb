use crate::error::{MigrationError, Result};
use crate::parser::{MigrationFile, MigrationParser};
use crate::tracker::MigrationTracker;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub version: String,
    pub name: String,
    pub applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigratorConfig {
    pub migrations_dir: PathBuf,
    pub schema_dir: PathBuf,
    pub auto_migrate: bool,
}

impl Default for MigratorConfig {
    fn default() -> Self {
        Self {
            migrations_dir: PathBuf::from("./migrations"),
            schema_dir: PathBuf::from("./schema"),
            auto_migrate: true,
        }
    }
}

pub struct Migrator {
    config: MigratorConfig,
    parser: MigrationParser,
    tracker: Arc<dyn MigrationTracker>,
}

impl Migrator {
    pub fn new(config: MigratorConfig, tracker: Arc<dyn MigrationTracker>) -> Self {
        Self {
            config,
            parser: MigrationParser::new(),
            tracker,
        }
    }

    pub fn run(&self) -> Result<Vec<MigrationFile>> {
        self.tracker.initialize()?;

        let applied_versions = self.tracker.get_applied_versions()?;
        let mut migrations = self.parser.parse_directory(&self.config.migrations_dir)?;

        let mut applied = Vec::new();

        for migration in &mut migrations {
            if applied_versions.contains(&migration.version) {
                migration.applied = true;
            } else {
                self.apply_migration(migration)?;
                migration.applied = true;
                applied.push(migration.clone());
            }
        }

        Ok(applied)
    }

    pub fn run_single(&self, version: &str) -> Result<MigrationFile> {
        self.tracker.initialize()?;

        if self.tracker.is_applied(version)? {
            return Err(MigrationError::AlreadyApplied(version.to_string()));
        }

        let migrations = self.parser.parse_directory(&self.config.migrations_dir)?;
        let mut migration = migrations
            .into_iter()
            .find(|m| m.version == version)
            .ok_or_else(|| MigrationError::NotFound(version.to_string()))?;

        self.apply_migration(&migration)?;
        migration.applied = true;
        Ok(migration)
    }

    fn apply_migration(&self, migration: &MigrationFile) -> Result<()> {
        log::info!(
            "Applying migration: {} ({})",
            migration.version,
            migration.name
        );

        self.tracker.record_migration(&migration.version)?;

        log::info!("Migration applied successfully: {}", migration.version);
        Ok(())
    }

    pub fn status(&self) -> Result<Vec<MigrationStatus>> {
        self.tracker.initialize()?;

        let applied_versions = self.tracker.get_applied_versions()?;
        let migrations = self.parser.parse_directory(&self.config.migrations_dir)?;

        let status: Vec<MigrationStatus> = migrations
            .into_iter()
            .map(|m| MigrationStatus {
                version: m.version.clone(),
                name: m.name.clone(),
                applied: applied_versions.contains(&m.version),
                applied_at: if applied_versions.contains(&m.version) {
                    Some(Utc::now())
                } else {
                    None
                },
            })
            .collect();

        Ok(status)
    }

    pub fn create_migration(&self, name: &str, content: &str) -> Result<PathBuf> {
        MigrationParser::create_migration_file(&self.config.migrations_dir, name, content)
    }

    pub fn remove_pending(&self) -> Result<Option<PathBuf>> {
        let applied_versions = self.tracker.get_applied_versions()?;
        let migrations = self.parser.parse_directory(&self.config.migrations_dir)?;

        if let Some(last) = migrations.last() {
            if applied_versions.contains(&last.version) {
                return Err(MigrationError::AlreadyApplied(format!(
                    "Cannot remove migration {} - already applied",
                    last.version
                )));
            }

            std::fs::remove_file(&last.path)?;
            Ok(Some(last.path.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn generate_models<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        schema_path: P,
        target_lang: &str,
        output_dir: Q,
        package_or_namespace: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        let lang = crate::codegen::TargetLanguage::parse(target_lang)?;
        crate::codegen::generate_models(schema_path, lang, output_dir, package_or_namespace)
    }

    pub fn diff_drawdb<P: AsRef<Path>>(
        &self,
        schema_path: P,
        migration_name: &str,
        dialect: Option<&str>,
        force_full: bool,
    ) -> Result<crate::codegen::DiffResult> {
        self.diff_drawdb_with_schema_dir(schema_path, None::<&Path>, migration_name, dialect, force_full)
    }

    pub fn diff_drawdb_with_schema_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        schema_path: P,
        schema_dir: Option<Q>,
        migration_name: &str,
        dialect: Option<&str>,
        force_full: bool,
    ) -> Result<crate::codegen::DiffResult> {
        let sch_dir = match &schema_dir {
            Some(d) => d.as_ref().to_path_buf(),
            None => self.config.schema_dir.clone(),
        };
        crate::codegen::diff_drawdb(
            schema_path,
            &self.config.migrations_dir,
            &sch_dir,
            migration_name,
            dialect,
            force_full,
        )
    }

    pub fn plan_sync_drawdb<P: AsRef<Path>>(
        &self,
        schema_path: P,
        dialect: Option<&str>,
        force_full: bool,
    ) -> Result<crate::codegen::SyncPlan> {
        self.plan_sync_drawdb_with_schema_dir(schema_path, None::<&Path>, dialect, force_full)
    }

    pub fn plan_sync_drawdb_with_schema_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        schema_path: P,
        schema_dir: Option<Q>,
        dialect: Option<&str>,
        force_full: bool,
    ) -> Result<crate::codegen::SyncPlan> {
        let sch_dir = match &schema_dir {
            Some(d) => d.as_ref().to_path_buf(),
            None => self.config.schema_dir.clone(),
        };
        crate::codegen::plan_sync_drawdb(
            schema_path,
            &self.config.migrations_dir,
            &sch_dir,
            dialect,
            force_full,
        )
    }

    pub fn get_pending(&self) -> Result<Vec<MigrationFile>> {
        let applied_versions = self.tracker.get_applied_versions()?;
        let migrations = self.parser.parse_directory(&self.config.migrations_dir)?;

        Ok(migrations
            .into_iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect())
    }

    pub fn get_applied(&self) -> Result<Vec<MigrationFile>> {
        let applied_versions = self.tracker.get_applied_versions()?;
        let migrations = self.parser.parse_directory(&self.config.migrations_dir)?;

        Ok(migrations
            .into_iter()
            .filter(|m| applied_versions.contains(&m.version))
            .collect())
    }
}

pub struct MigratorBuilder {
    config: MigratorConfig,
}

impl MigratorBuilder {
    pub fn new() -> Self {
        Self {
            config: MigratorConfig::default(),
        }
    }

    pub fn migrations_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.config.migrations_dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn schema_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.config.schema_dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn auto_migrate(mut self, auto: bool) -> Self {
        self.config.auto_migrate = auto;
        self
    }

    pub fn build(self, tracker: Arc<dyn MigrationTracker>) -> Migrator {
        Migrator::new(self.config, tracker)
    }
}

impl Default for MigratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
