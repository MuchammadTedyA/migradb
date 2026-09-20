use pyo3::prelude::*;
use migration_engine::{MigratorBuilder, InMemoryTracker, MigrationStatus as CoreStatus};
use std::sync::Arc;

#[pyclass]
#[derive(Clone, Debug)]
struct Migration {
    #[pyo3(get)]
    version: String,
    #[pyo3(get)]
    name: String,
}

#[pyclass]
#[derive(Clone, Debug)]
struct MigrationStatus {
    #[pyo3(get)]
    version: String,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    applied: bool,
    #[pyo3(get)]
    applied_at: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct RunResult {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    applied: usize,
    #[pyo3(get)]
    migrations: Vec<Migration>,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct StatusResult {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    migrations: Vec<MigrationStatus>,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct CreateResult {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    path: String,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct RemoveResult {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    removed: Option<String>,
    #[pyo3(get)]
    message: Option<String>,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct GenerateResult {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    files: Vec<String>,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct DiffResult {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    migration_path: Option<String>,
    #[pyo3(get)]
    diff_summary: String,
    #[pyo3(get)]
    is_empty: bool,
    #[pyo3(get)]
    statements: Vec<String>,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct SyncPlan {
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    is_empty: bool,
    #[pyo3(get)]
    diff_summary: String,
    #[pyo3(get)]
    statements: Vec<String>,
    #[pyo3(get)]
    sql: String,
    #[pyo3(get)]
    dialect: String,
    #[pyo3(get)]
    error: Option<String>,
}

#[pyfunction]
#[pyo3(signature = (schema_path, migrations_dir=None, schema_dir=None, migration_name=None, dialect=None, force_full=false))]
fn diff_drawdb(
    schema_path: &str,
    migrations_dir: Option<&str>,
    schema_dir: Option<&str>,
    migration_name: Option<&str>,
    dialect: Option<&str>,
    force_full: bool,
) -> DiffResult {
    let mig_dir = migrations_dir.unwrap_or("./migrations");
    let sch_dir = schema_dir.unwrap_or("./schema");
    let name = migration_name.unwrap_or("sync_drawdb");

    match migration_engine::codegen::diff_drawdb(
        schema_path,
        mig_dir,
        sch_dir,
        name,
        dialect,
        force_full,
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
            diff_summary: String::new(),
            is_empty: false,
            statements: Vec::new(),
            error: Some(e.to_string()),
        },
    }
}

#[pyfunction]
#[pyo3(signature = (schema_path, migrations_dir=None, schema_dir=None, dialect=None, force_full=false))]
fn plan_sync_drawdb(
    schema_path: &str,
    migrations_dir: Option<&str>,
    schema_dir: Option<&str>,
    dialect: Option<&str>,
    force_full: bool,
) -> SyncPlan {
    let mig_dir = migrations_dir.unwrap_or("./migrations");
    let sch_dir = schema_dir.unwrap_or("./schema");

    match migration_engine::codegen::plan_sync_drawdb(
        schema_path,
        mig_dir,
        sch_dir,
        dialect,
        force_full,
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
            diff_summary: String::new(),
            statements: Vec::new(),
            sql: String::new(),
            dialect: String::new(),
            error: Some(e.to_string()),
        },
    }
}

#[pyfunction]
#[pyo3(signature = (schema_path, target_lang, output_dir, pkg_or_namespace=None))]
fn generate_models(
    schema_path: &str,
    target_lang: &str,
    output_dir: &str,
    pkg_or_namespace: Option<&str>,
) -> GenerateResult {
    let lang = match migration_engine::codegen::TargetLanguage::parse(target_lang) {
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
        schema_path,
        lang,
        output_dir,
        pkg_or_namespace,
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

#[pyclass]
struct Migrator {
    inner: migration_engine::Migrator,
}

#[pymethods]
impl Migrator {
    #[new]
    fn new(migrations_dir: &str) -> Self {
        let tracker = Arc::new(InMemoryTracker::new());
        let migrator = MigratorBuilder::new()
            .migrations_dir(migrations_dir)
            .build(tracker);

        Self { inner: migrator }
    }

    #[pyo3(signature = (schema_path, target_lang, output_dir, pkg_or_namespace=None))]
    fn generate_models(
        &self,
        schema_path: &str,
        target_lang: &str,
        output_dir: &str,
        pkg_or_namespace: Option<&str>,
    ) -> GenerateResult {
        generate_models(schema_path, target_lang, output_dir, pkg_or_namespace)
    }

    fn run(&self) -> RunResult {
        match self.inner.run() {
            Ok(migrations) => RunResult {
                success: true,
                applied: migrations.len(),
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

    fn status(&self) -> StatusResult {
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

    fn create(&self, name: &str, content: &str) -> CreateResult {
        match self.inner.create_migration(name, content) {
            Ok(path) => CreateResult {
                success: true,
                path: path.to_string_lossy().to_string(),
                error: None,
            },
            Err(e) => CreateResult {
                success: false,
                path: String::new(),
                error: Some(e.to_string()),
            },
        }
    }

    fn remove_pending(&self) -> RemoveResult {
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

    #[pyo3(signature = (schema_path, migration_name=None, dialect=None, force_full=false))]
    fn diff_drawdb(
        &self,
        schema_path: &str,
        migration_name: Option<&str>,
        dialect: Option<&str>,
        force_full: bool,
    ) -> DiffResult {
        let name = migration_name.unwrap_or("sync_drawdb");
        match self.inner.diff_drawdb(schema_path, name, dialect, force_full) {
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
                diff_summary: String::new(),
                is_empty: false,
                statements: Vec::new(),
                error: Some(e.to_string()),
            },
        }
    }

    #[pyo3(signature = (schema_path, dialect=None, force_full=false))]
    fn plan_sync_drawdb(
        &self,
        schema_path: &str,
        dialect: Option<&str>,
        force_full: bool,
    ) -> SyncPlan {
        match self.inner.plan_sync_drawdb(schema_path, dialect, force_full) {
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
                diff_summary: String::new(),
                statements: Vec::new(),
                sql: String::new(),
                dialect: String::new(),
                error: Some(e.to_string()),
            },
        }
    }
}

#[pymodule]
fn migration_engine_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Migrator>()?;
    m.add_class::<Migration>()?;
    m.add_class::<MigrationStatus>()?;
    m.add_class::<RunResult>()?;
    m.add_class::<StatusResult>()?;
    m.add_class::<CreateResult>()?;
    m.add_class::<RemoveResult>()?;
    m.add_class::<GenerateResult>()?;
    m.add_class::<DiffResult>()?;
    m.add_class::<SyncPlan>()?;
    m.add_function(wrap_pyfunction!(generate_models, m)?)?;
    m.add_function(wrap_pyfunction!(diff_drawdb, m)?)?;
    m.add_function(wrap_pyfunction!(plan_sync_drawdb, m)?)?;
    Ok(())
}
