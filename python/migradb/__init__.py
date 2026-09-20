"""
MigrDB for Python

A SQL database migration library with Rust engine.
"""

try:
    from .migration_engine_py import (
        Migrator,
        Migration,
        MigrationStatus,
        RunResult,
        StatusResult,
        CreateResult,
        RemoveResult,
        GenerateResult,
        DiffResult,
        SyncPlan,
        generate_models,
        diff_drawdb,
        plan_sync_drawdb,
    )
except ImportError:
    from migration_engine_py import (
        Migrator,
        Migration,
        MigrationStatus,
        RunResult,
        StatusResult,
        CreateResult,
        RemoveResult,
        GenerateResult,
        DiffResult,
        SyncPlan,
        generate_models,
        diff_drawdb,
        plan_sync_drawdb,
    )
from .runner import run_on_connection, status_on_connection, sync_database, detect_dialect

__version__ = "0.1.2"
__all__ = [
    "Migrator",
    "Migration",
    "MigrationStatus",
    "RunResult",
    "StatusResult",
    "CreateResult",
    "RemoveResult",
    "GenerateResult",
    "DiffResult",
    "SyncPlan",
    "generate_models",
    "diff_drawdb",
    "plan_sync_drawdb",
    "run_on_connection",
    "status_on_connection",
    "sync_database",
    "detect_dialect",
]
