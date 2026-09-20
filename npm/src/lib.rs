use napi_derive::napi;
use migration_engine::{MigratorBuilder, InMemoryTracker, MigrationStatus as CoreStatus};
use std::sync::Arc;

#[napi(object)]
pub struct Migration {
    pub version: String,
    pub name: String,
}

#[napi(object)]
pub struct MigrationStatus {
    pub version: String,
    pub name: String,
    pub applied: bool,
    pub applied_at: Option<String>,
}

#[napi(object)]
pub struct RunResult {
    pub success: bool,
    pub applied: i32,
    pub migrations: Vec<Migration>,
    pub error: Option<String>,
}

#[napi(object)]
pub struct StatusResult {
    pub success: bool,
    pub migrations: Vec<MigrationStatus>,
    pub error: Option<String>,
}

#[napi(object)]
pub struct CreateResult {
    pub success: bool,
    pub path: String,
    pub error: Option<String>,
}

#[napi(object)]
pub struct RemoveResult {
    pub success: bool,
    pub removed: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[napi(object)]
pub struct GenerateResult {
    pub success: bool,
    pub files: Vec<String>,
    pub error: Option<String>,
}

#[napi(object)]
pub struct DiffResult {
    pub success: bool,
    pub migration_path: Option<String>,
    pub diff_summary: String,
    pub is_empty: bool,
    pub statements: Vec<String>,
    pub error: Option<String>,
}

#[napi(object)]
pub struct SyncPlan {
    pub success: bool,
    pub is_empty: bool,
    pub diff_summary: String,
    pub statements: Vec<String>,
    pub sql: String,
    pub dialect: String,
    pub error: Option<String>,
}

#[napi]
pub fn diff_drawdb(
    schema_path: String,
    migrations_dir: Option<String>,
    schema_dir: Option<String>,
    migration_name: Option<String>,
    dialect: Option<String>,
    force_full: Option<bool>,
) -> DiffResult {
    let mig_dir = migrations_dir.unwrap_or_else(|| "./migrations".to_string());
    let sch_dir = schema_dir.unwrap_or_else(|| "./schema".to_string());
    let name = migration_name.unwrap_or_else(|| "sync_drawdb".to_string());

    match migration_engine::codegen::diff_drawdb(
        &schema_path,
        &mig_dir,
        &sch_dir,
        &name,
        dialect.as_deref(),
        force_full.unwrap_or(false),
    ) {
        Ok(res) => DiffResult {
            success: res.success,
            migration_path: res.migration_path,
            diff_summary: res.diff_summary,
            is_empty: res.is_empty,
            statements: res.statements,
            error: res.error,
        },
        Err(e) => DiffResult {
            success: false,
            migration_path: None,
            diff_summary: "".to_string(),
            is_empty: false,
            statements: vec![],
            error: Some(e.to_string()),
        },
    }
}

#[napi]
pub fn plan_sync_drawdb(
    schema_path: String,
    migrations_dir: Option<String>,
    schema_dir: Option<String>,
    dialect: Option<String>,
    force_full: Option<bool>,
) -> SyncPlan {
    let mig_dir = migrations_dir.unwrap_or_else(|| "./migrations".to_string());
    let sch_dir = schema_dir.unwrap_or_else(|| "./schema".to_string());

    match migration_engine::codegen::plan_sync_drawdb(
        &schema_path,
        &mig_dir,
        &sch_dir,
        dialect.as_deref(),
        force_full.unwrap_or(false),
    ) {
        Ok(res) => SyncPlan {
            success: res.success,
            is_empty: res.is_empty,
            diff_summary: res.diff_summary,
            statements: res.statements,
            sql: res.sql,
            dialect: res.dialect,
            error: res.error,
        },
        Err(e) => SyncPlan {
            success: false,
            is_empty: false,
            diff_summary: "".to_string(),
            statements: vec![],
            sql: "".to_string(),
            dialect: "".to_string(),
            error: Some(e.to_string()),
        },
    }
}

#[napi]
pub fn generate_models(
    schema_path: String,
    target_lang: String,
    output_dir: String,
    pkg_or_namespace: Option<String>,
) -> GenerateResult {
    let lang = match migration_engine::codegen::TargetLanguage::parse(&target_lang) {
        Ok(l) => l,
        Err(e) => {
            return GenerateResult {
                success: false,
                files: vec![],
                error: Some(e.to_string()),
            };
        }
    };

    match migration_engine::codegen::generate_models(
        &schema_path,
        lang,
        &output_dir,
        pkg_or_namespace.as_deref(),
    ) {
        Ok(paths) => GenerateResult {
            success: true,
            files: paths
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            error: None,
        },
        Err(e) => GenerateResult {
            success: false,
            files: vec![],
            error: Some(e.to_string()),
        },
    }
}

#[napi]
pub struct Migrator {
    inner: migration_engine::Migrator,
}

#[napi]
impl Migrator {
    #[napi(constructor)]
    pub fn new(migrations_dir: String) -> Self {
        let tracker = Arc::new(InMemoryTracker::new());
        let migrator = MigratorBuilder::new()
            .migrations_dir(&migrations_dir)
            .build(tracker);

        Self { inner: migrator }
    }

    #[napi]
    pub fn generate_models(
        &self,
        schema_path: String,
        target_lang: String,
        output_dir: String,
        pkg_or_namespace: Option<String>,
    ) -> GenerateResult {
        generate_models(schema_path, target_lang, output_dir, pkg_or_namespace)
    }

    #[napi]
    pub fn run(&self) -> RunResult {
        match self.inner.run() {
            Ok(migrations) => RunResult {
                success: true,
                applied: migrations.len() as i32,
                migrations: migrations
                    .into_iter()
                    .map(|m| Migration {
                        version: m.version,
                        name: m.name,
                    })
                    .collect(),
                error: None,
            },
            Err(e) => RunResult {
                success: false,
                applied: 0,
                migrations: vec![],
                error: Some(e.to_string()),
            },
        }
    }

    #[napi]
    pub fn status(&self) -> StatusResult {
        match self.inner.status() {
            Ok(status_list) => StatusResult {
                success: true,
                migrations: status_list
                    .into_iter()
                    .map(|s: CoreStatus| MigrationStatus {
                        version: s.version,
                        name: s.name,
                        applied: s.applied,
                        applied_at: s.applied_at.map(|dt| dt.to_rfc3339()),
                    })
                    .collect(),
                error: None,
            },
            Err(e) => StatusResult {
                success: false,
                migrations: vec![],
                error: Some(e.to_string()),
            },
        }
    }

    #[napi]
    pub fn create(&self, name: String, content: String) -> CreateResult {
        match self.inner.create_migration(&name, &content) {
            Ok(path) => CreateResult {
                success: true,
                path: path.to_string_lossy().to_string(),
                error: None,
            },
            Err(e) => CreateResult {
                success: false,
                path: "".to_string(),
                error: Some(e.to_string()),
            },
        }
    }

    #[napi]
    pub fn remove_pending(&self) -> RemoveResult {
        match self.inner.remove_pending() {
            Ok(Some(path)) => RemoveResult {
                success: true,
                removed: Some(path.to_string_lossy().to_string()),
                message: None,
                error: None,
            },
            Ok(None) => RemoveResult {
                success: true,
                removed: None,
                message: Some("No pending migrations to remove".to_string()),
                error: None,
            },
            Err(e) => RemoveResult {
                success: false,
                removed: None,
                message: None,
                error: Some(e.to_string()),
            },
        }
    }

    #[napi]
    pub fn diff_drawdb(
        &self,
        schema_path: String,
        migration_name: Option<String>,
        dialect: Option<String>,
        force_full: Option<bool>,
    ) -> DiffResult {
        let name = migration_name.unwrap_or_else(|| "sync_drawdb".to_string());
        match self.inner.diff_drawdb(
            &schema_path,
            &name,
            dialect.as_deref(),
            force_full.unwrap_or(false),
        ) {
            Ok(res) => DiffResult {
                success: res.success,
                migration_path: res.migration_path,
                diff_summary: res.diff_summary,
                is_empty: res.is_empty,
                statements: res.statements,
                error: res.error,
            },
            Err(e) => DiffResult {
                success: false,
                migration_path: None,
                diff_summary: "".to_string(),
                is_empty: false,
                statements: vec![],
                error: Some(e.to_string()),
            },
        }
    }

    #[napi]
    pub fn plan_sync_drawdb(
        &self,
        schema_path: String,
        dialect: Option<String>,
        force_full: Option<bool>,
    ) -> SyncPlan {
        match self.inner.plan_sync_drawdb(
            &schema_path,
            dialect.as_deref(),
            force_full.unwrap_or(false),
        ) {
            Ok(res) => SyncPlan {
                success: res.success,
                is_empty: res.is_empty,
                diff_summary: res.diff_summary,
                statements: res.statements,
                sql: res.sql,
                dialect: res.dialect,
                error: res.error,
            },
            Err(e) => SyncPlan {
                success: false,
                is_empty: false,
                diff_summary: "".to_string(),
                statements: vec![],
                sql: "".to_string(),
                dialect: "".to_string(),
                error: Some(e.to_string()),
            },
        }
    }
}
