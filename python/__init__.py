"""
MigrDB for Python

A SQL database migration library with Rust engine.
"""

from migration_engine_py import (
    Migrator,
    Migration,
    MigrationStatus,
    RunResult,
    StatusResult,
    CreateResult,
    RemoveResult,
    GenerateResult,
    generate_models,
)
from .runner import run_on_connection, status_on_connection

__version__ = "0.1.1"
__all__ = [
    "Migrator",
    "Migration",
    "MigrationStatus",
    "RunResult",
    "StatusResult",
    "CreateResult",
    "RemoveResult",
    "GenerateResult",
    "generate_models",
    "run_on_connection",
    "status_on_connection",
]
