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
)
from .runner import run_on_connection, status_on_connection

__version__ = "0.1.0"
__all__ = [
    "Migrator",
    "Migration",
    "MigrationStatus",
    "RunResult",
    "StatusResult",
    "CreateResult",
    "RemoveResult",
    "run_on_connection",
    "status_on_connection",
]
