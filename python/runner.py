import os
import re
from typing import Any, Dict, List, Optional, Set

VERSION_REGEX = re.compile(r"^(\d+)(?:_(.+))?\.sql$")


class ResultDict(dict):
    """A dictionary that also allows attribute access (e.g. res.success, res.migrations)."""
    def __getattr__(self, key: str) -> Any:
        try:
            return self[key]
        except KeyError:
            raise AttributeError(f"'ResultDict' object has no attribute '{key}'")

    def __setattr__(self, key: str, value: Any) -> None:
        self[key] = value


def run_on_connection(conn: Any, migrations_dir: str) -> ResultDict:
    """
    Applies all pending SQL migrations found in migrations_dir against any standard
    DB-API 2.0 database connection (e.g. sqlite3, psycopg2, pymysql).
    
    Each migration is executed within a transaction, and the schema_migrations table
    is automatically created and updated.
    """
    try:
        cursor = conn.cursor()

        # 1. Initialize tracking table
        init_sql = """
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        """
        cursor.execute(init_sql)
        conn.commit()

        # 2. Query applied migrations
        cursor.execute("SELECT version FROM schema_migrations;")
        applied_set: Set[str] = {str(row[0]) for row in cursor.fetchall()}

        # 3. Read migration files
        if not os.path.exists(migrations_dir):
            raise FileNotFoundError(f"Migrations directory not found: {migrations_dir}")

        entries = []
        for filename in os.listdir(migrations_dir):
            if not filename.endswith(".sql"):
                continue
            match = VERSION_REGEX.match(filename)
            if match:
                version = match.group(1)
                name = match.group(2) or filename[:-4]
                entries.append((version, name, filename))

        entries.sort(key=lambda x: x[0])

        applied: List[ResultDict] = []
        for version, name, filename in entries:
            if version in applied_set:
                continue

            file_path = os.path.join(migrations_dir, filename)
            with open(file_path, "r", encoding="utf-8") as f:
                sql_content = f.read()

            try:
                if sql_content.strip():
                    cursor.execute(sql_content)

                record_sql = f"INSERT INTO schema_migrations (version) VALUES ('{version}');"
                cursor.execute(record_sql)
                conn.commit()

                applied.append(ResultDict({"version": version, "name": name}))
            except Exception as e:
                conn.rollback()
                raise e

        return ResultDict({
            "success": True,
            "applied": len(applied),
            "migrations": applied,
            "error": None,
        })
    except Exception as e:
        return ResultDict({
            "success": False,
            "applied": 0,
            "migrations": [],
            "error": str(e),
        })


def status_on_connection(conn: Any, migrations_dir: str) -> ResultDict:
    """
    Inspects migration status (applied vs pending) against any standard DB-API 2.0 connection.
    """
    try:
        cursor = conn.cursor()
        init_sql = """
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        """
        cursor.execute(init_sql)
        conn.commit()

        cursor.execute("SELECT version FROM schema_migrations;")
        applied_set: Set[str] = {str(row[0]) for row in cursor.fetchall()}

        if not os.path.exists(migrations_dir):
            raise FileNotFoundError(f"Migrations directory not found: {migrations_dir}")

        entries = []
        for filename in os.listdir(migrations_dir):
            if not filename.endswith(".sql"):
                continue
            match = VERSION_REGEX.match(filename)
            if match:
                version = match.group(1)
                name = match.group(2) or filename[:-4]
                entries.append(ResultDict({
                    "version": version,
                    "name": name,
                    "applied": version in applied_set,
                    "applied_at": None,
                }))

        entries.sort(key=lambda x: x["version"])

        return ResultDict({
            "success": True,
            "migrations": entries,
            "error": None,
        })
    except Exception as e:
        return ResultDict({
            "success": False,
            "migrations": [],
            "error": str(e),
        })


def detect_dialect(conn: Any) -> str:
    """
    Detects the SQL dialect ('postgres', 'mysql', or 'sqlite') based on the connection type.
    Defaults to 'postgres' if unknown.
    """
    if conn is None:
        return "postgres"
    conn_type = f"{type(conn).__module__}.{type(conn).__name__}".lower()
    if "sqlite" in conn_type:
        return "sqlite"
    if "mysql" in conn_type:
        return "mysql"
    return "postgres"


def _get_engine_funcs():
    try:
        from .migration_engine_py import diff_drawdb, plan_sync_drawdb
        return diff_drawdb, plan_sync_drawdb
    except (ImportError, ValueError):
        try:
            from migradb.migration_engine_py import diff_drawdb, plan_sync_drawdb
            return diff_drawdb, plan_sync_drawdb
        except ImportError:
            from migration_engine_py import diff_drawdb, plan_sync_drawdb
            return diff_drawdb, plan_sync_drawdb


def sync_database(
    conn: Any,
    schema: str = "./schema/drawdb.json",
    migrations_dir: str = "./migrations",
    schema_dir: str = "./schema",
    dialect: Optional[str] = None,
    save_migration_file: bool = True,
    migration_name: str = "sync_drawdb",
    force_full: bool = False,
) -> ResultDict:
    """
    Synchronizes the database schema against a DrawDB schema file.
    Dynamically compares schema against snapshot, translates DDL for target dialect,
    executes in a transaction, updates schema_migrations and snapshot, and optionally saves audit SQL.
    """
    import datetime

    target_dialect = dialect or detect_dialect(conn)

    os.makedirs(schema_dir, exist_ok=True)
    os.makedirs(migrations_dir, exist_ok=True)

    diff_drawdb, plan_sync_drawdb = _get_engine_funcs()

    plan = plan_sync_drawdb(
        schema_path=schema,
        migrations_dir=migrations_dir,
        schema_dir=schema_dir,
        dialect=target_dialect,
        force_full=force_full,
    )

    if not plan.success and plan.error:
        return ResultDict({
            "success": False,
            "is_empty": False,
            "applied": 0,
            "diff_summary": "",
            "migration_path": None,
            "statements": [],
            "error": plan.error,
        })

    if plan.is_empty or not plan.statements:
        return ResultDict({
            "success": True,
            "is_empty": True,
            "applied": 0,
            "diff_summary": plan.diff_summary,
            "statements": [],
            "migration_path": None,
            "error": None,
        })

    try:
        cursor = conn.cursor()

        init_sql = """
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        """
        cursor.execute(init_sql)
        conn.commit()

        # Execute statements
        for stmt in plan.statements:
            s = stmt.strip()
            if s:
                cursor.execute(s)

        now = datetime.datetime.utcnow().strftime("%Y%m%d%H%M%S")
        record_sql = f"INSERT INTO schema_migrations (version) VALUES ('{now}');"
        cursor.execute(record_sql)
        conn.commit()

        migration_path = None
        if save_migration_file:
            diff_res = diff_drawdb(
                schema_path=schema,
                migrations_dir=migrations_dir,
                schema_dir=schema_dir,
                migration_name=migration_name,
                dialect=target_dialect,
                force_full=force_full,
            )
            if diff_res and diff_res.migration_path:
                migration_path = diff_res.migration_path
        else:
            with open(schema, "r", encoding="utf-8") as f:
                content = f.read()
            with open(os.path.join(schema_dir, "drawdb_snapshot.json"), "w", encoding="utf-8") as f:
                f.write(content)
            with open(os.path.join(migrations_dir, ".schema_snapshot.json"), "w", encoding="utf-8") as f:
                f.write(content)

        return ResultDict({
            "success": True,
            "is_empty": False,
            "applied": len(plan.statements),
            "diff_summary": plan.diff_summary,
            "migration_path": migration_path,
            "statements": list(plan.statements),
            "error": None,
        })
    except Exception as e:
        try:
            conn.rollback()
        except Exception:
            pass
        return ResultDict({
            "success": False,
            "is_empty": False,
            "applied": 0,
            "diff_summary": plan.diff_summary,
            "migration_path": None,
            "statements": list(plan.statements),
            "error": str(e),
        })

