import os
import re
from typing import Any, Dict, List, Optional, Set

VERSION_REGEX = re.compile(r"^(\d+)(?:_(.+))?\.sql$")


def run_on_connection(conn: Any, migrations_dir: str) -> Dict[str, Any]:
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

        applied: List[Dict[str, str]] = []
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

                applied.append({"version": version, "name": name})
            except Exception as e:
                conn.rollback()
                raise e

        return {
            "success": True,
            "applied": len(applied),
            "migrations": applied,
            "error": None,
        }
    except Exception as e:
        return {
            "success": False,
            "applied": 0,
            "migrations": [],
            "error": str(e),
        }


def status_on_connection(conn: Any, migrations_dir: str) -> Dict[str, Any]:
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
                entries.append({
                    "version": version,
                    "name": name,
                    "applied": version in applied_set,
                })

        entries.sort(key=lambda x: x["version"])

        return {
            "success": True,
            "migrations": entries,
            "error": None,
        }
    except Exception as e:
        return {
            "success": False,
            "migrations": [],
            "error": str(e),
        }
