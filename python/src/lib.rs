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
    Ok(())
}
