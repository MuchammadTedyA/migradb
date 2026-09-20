# Python Guide

Complete guide for using MigrDB in Python projects.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Integration with Frameworks](#integration-with-frameworks)
- [Async Support](#async-support)
- [Generated SQLAlchemy Models](#generated-sqlalchemy-models)

## Installation

### Pip

```bash
pip install migradb
```

### Poetry

```bash
poetry add migradb
```

### Pipenv

```bash
pipenv install migradb
```

## Quick Start

### 1. Universal Schema Sync (`sync_database`) - Recommended

Design your database schema visually in DrawDB, export the JSON to `./schema/drawdb.json`, and let MigraDB dynamically synchronize your live database:

```python
import psycopg2
from migradb import sync_database

conn = psycopg2.connect("dbname=mydb user=postgres password=secret host=localhost")

# Synchronize schema against drawdb.json
res = sync_database(
    conn=conn,
    schema="./schema/drawdb.json",    # Default schema path
    migrations_dir="./migrations",     # Auto-created if missing
    schema_dir="./schema",             # Auto-created if missing
    save_migration_file=True,          # Generates audit .sql migration
    migration_name="sync_schema",      # Migration slug
)

if res.is_empty:
    print("✅ Schema is already up to date.")
elif res.success:
    print(f"✅ Applied {res.applied} statements! Audit file: {res.migration_path}")
else:
    print(f"❌ Sync failed: {res.error}")

conn.close()
```

#### Why Approach 1?
- **Zero Migration Regeneration**: If you change your database midway (e.g. SQLite during local development -> PostgreSQL or MySQL in production), **you do not need to rewrite or regenerate any migrations**. Simply connect to the new database, and MigraDB translates the schema diff into the correct DDL dialect on the fly!
- **Auto Directory Conventions**: Automatically creates `./schema` and `./migrations` in the root project if they do not exist. If they already exist, files are placed directly inside.
- **Dialect Defaults**: Defaults to **PostgreSQL**. MySQL and SQLite are fully supported (specify via `dialect="mysql"` / `"postgres"` / `"sqlite"` or let MigraDB auto-detect from connection).
- **Audit Logging**: Automatically writes timestamped audit migrations (`migrations/YYYYMMDDHHmmss_sync_drawdb.sql`) and maintains snapshots (`schema/drawdb_snapshot.json`).

### 2. Running Manual Migration Files (`run_on_connection`)

MigraDB works with any standard Python DB-API 2.0 connection (`sqlite3`, `psycopg2`, `pymysql`):


```python
import psycopg2
from migradb import run_on_connection, status_on_connection

conn = psycopg2.connect("dbname=mydb user=postgres password=secret host=localhost")

# Check migration status
status = status_on_connection(conn, "./migrations")
print("Status:", status["migrations"])

# Run pending migrations in transactions
result = run_on_connection(conn, "./migrations")
if result["success"]:
    print(f"Applied {result['applied']} migrations!")
else:
    print(f"Error: {result['error']}")

conn.close()
```

### 2. Generating Migrations & Models from DrawDB (`drawdb.json`)

```python
from migradb import generate_models

# Generate PostgreSQL migration files from DrawDB:
generate_models("schema/drawdb.json", "sql", "./migrations", "postgres")

# Generate Python dataclass models from the same diagram:
generate_models("schema/drawdb.json", "python", "./models")
```

### 3. In-Memory Native Core Engine (`Migrator`)

For validating migration files and timestamps using the native Rust engine:

```python
from migradb import Migrator

m = Migrator("./migrations")
status = m.status()
result = m.run()
```

## API Reference

### Types

#### `Migrator`

The main class for managing migrations.

```python
class Migrator:
    def __init__(self, migrations_dir: str) -> None: ...
    def run(self) -> RunResult: ...
    def status(self) -> StatusResult: ...
    def create(self, name: str, content: str) -> CreateResult: ...
    def remove_pending(self) -> RemoveResult: ...
```

#### `Migration`

Represents a single migration.

```python
class Migration:
    version: str
    name: str
```

#### `MigrationStatus`

Represents the status of a migration.

```python
class MigrationStatus:
    version: str
    name: str
    applied: bool
    applied_at: Optional[str]
```

#### `RunResult`

Result from running migrations.

```python
class RunResult:
    success: bool
    applied: int
    migrations: List[Migration]
    error: Optional[str]
```

#### `StatusResult`

Result from checking migration status.

```python
class StatusResult:
    success: bool
    migrations: List[MigrationStatus]
    error: Optional[str]
```

#### `CreateResult`

Result from creating a migration.

```python
class CreateResult:
    success: bool
    path: str
    error: Optional[str]
```

#### `RemoveResult`

Result from removing a pending migration.

```python
class RemoveResult:
    success: bool
    removed: Optional[str]
    message: Optional[str]
    error: Optional[str]
```

### Methods

#### `__init__(migrations_dir: str)`

Creates a new Migrator instance.

**Parameters:**
- `migrations_dir` - Path to the migrations directory

**Example:**
```python
m = Migrator("./migrations")
```

#### `run() -> RunResult`

Applies all pending migrations.

**Returns:**
- `RunResult` - Result of the operation

**Example:**
```python
result = m.run()
if result.success:
    print(f"Applied {result.applied} migrations")
```

#### `status() -> StatusResult`

Returns the status of all migrations.

**Returns:**
- `StatusResult` - Status of all migrations

**Example:**
```python
status = m.status()
for s in status.migrations:
    status_str = "APPLIED" if s.applied else "PENDING"
    print(f"[{status_str}] {s.version} - {s.name}")
```

#### `create(name: str, content: str) -> CreateResult`

Creates a new migration file.

**Parameters:**
- `name` - Human-readable name for the migration
- `content` - SQL content of the migration

**Returns:**
- `CreateResult` - Result of the operation

**Example:**
```python
result = m.create("add products table", """
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL
    );
""")
```

#### `remove_pending() -> RemoveResult`

Removes the last pending migration file.

**Returns:**
- `RemoveResult` - Result of the operation

**Example:**
```python
result = m.remove_pending()
if result.removed:
    print(f"Removed: {result.removed}")
```

#### `generate_models(schema_path: str, target_lang: str, output_dir: str, pkg_or_namespace: Optional[str] = None) -> GenerateResult`

Generates strongly typed models or SQL DDL from a DrawDB diagram export file. Available both as a standalone function `from migradb import generate_models` and as a `Migrator` method `m.generate_models(...)`.

**Parameters:**
- `schema_path` - Path to the DrawDB JSON export file
- `target_lang` - Target language (`"python"`, `"py"`, `"node"`, `"go"`, `"rust"`, `"csharp"`, `"sql"`)
- `output_dir` - Destination file path (e.g. `"./models.py"`) or directory
- `pkg_or_namespace` - Optional package or namespace name

**Example:**
```python
from migradb import generate_models

res = generate_models("schema/drawdb.json", "python", "./app/models.py")
print("Generated files:", res.files)
```

#### `run_on_connection(conn: Any, migrations_dir: str) -> ResultDict`

Executes all pending migrations directly against any standard Python DB-API 2.0 database connection (`sqlite3`, `psycopg2`, `pymysql`) within atomic transactions. Automatically creates and updates the `schema_migrations` tracking table.

**Example:**
```python
import psycopg2
from migradb import run_on_connection

conn = psycopg2.connect("dbname=myapp user=postgres password=secret host=localhost")
result = run_on_connection(conn, "./migrations")
if result.success:
    print(f"Applied {result.applied} migrations")
conn.close()
```

#### `status_on_connection(conn: Any, migrations_dir: str) -> ResultDict`

Inspects migration status directly against a live database connection by checking the `schema_migrations` audit table.

**Example:**
```python
import psycopg2
from migradb import status_on_connection

conn = psycopg2.connect("dbname=myapp user=postgres password=secret host=localhost")
status = status_on_connection(conn, "./migrations")
for s in status.migrations:
    print(f"[{'APPLIED' if s.applied else 'PENDING'}] {s.version} - {s.name}")
conn.close()
```

## Examples

### Basic Migration Workflow

```python
from migradb import Migrator

m = Migrator("./migrations")

# Check current status
status = m.status()
if status.success:
    print("Current migration status:")
    for s in status.migrations:
        status_str = "APPLIED" if s.applied else "PENDING"
        print(f"  [{status_str}] {s.version} - {s.name}")

# Run pending migrations
result = m.run()
if result.success:
    print(f"\nApplied {result.applied} migrations:")
    for mig in result.migrations:
        print(f"  - {mig.version}: {mig.name}")
```

### Creating a New Migration

```python
result = m.create("add employees table", """
    CREATE TABLE IF NOT EXISTS m_employees (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES m_users(id),
        branch_id UUID REFERENCES m_branches(id),
        employee_code VARCHAR(20) NOT NULL UNIQUE,
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        hire_date DATE NOT NULL,
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_employees_user ON m_employees(user_id);
    CREATE INDEX IF NOT EXISTS idx_employees_branch ON m_employees(branch_id);
    CREATE INDEX IF NOT EXISTS idx_employees_code ON m_employees(employee_code);
""")

if result.success:
    print(f"Created migration at: {result.path}")
```

### CLI Migration Tool

```python
#!/usr/bin/env python3
import sys
from migradb import Migrator

def main():
    migrations_dir = sys.argv[1] if len(sys.argv) > 1 else "./migrations"
    command = sys.argv[2] if len(sys.argv) > 2 else "status"

    m = Migrator(migrations_dir)

    if command == "run":
        result = m.run()
        if result.success:
            print(f"Applied {result.applied} migrations")
        else:
            print(result.error, file=sys.stderr)
            sys.exit(1)

    elif command == "status":
        status = m.status()
        if status.success:
            for s in status.migrations:
                status_str = "APPLIED" if s.applied else "PENDING"
                print(f"[{status_str}] {s.version} - {s.name}")

    elif command == "create":
        name = sys.argv[3] if len(sys.argv) > 3 else None
        if not name:
            print("Usage: migrate <dir> create <name>", file=sys.stderr)
            sys.exit(1)
        result = m.create(name, "-- Add your SQL here\n")
        if result.success:
            print(f"Created: {result.path}")

    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        print("Commands: run, status, create", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## Integration with Frameworks

### Flask (Startup Auto-Migration)

```python
import os
import psycopg2
from flask import Flask, jsonify, request
from migradb import run_on_connection, status_on_connection, Migrator

app = Flask(__name__)
DB_URI = os.getenv("DATABASE_URL", "dbname=myapp user=postgres password=secret host=localhost")
migrator = Migrator("./migrations")

# Run migrations automatically on server startup
with app.app_context():
    conn = psycopg2.connect(DB_URI)
    try:
        result = run_on_connection(conn, "./migrations")
        if not result.success:
            raise RuntimeError(f"Startup migration failed: {result.error}")
        print(f"Applied {result.applied} startup migrations")
    finally:
        conn.close()

@app.route("/api/migrations/status")
def migration_status():
    conn = psycopg2.connect(DB_URI)
    try:
        status = status_on_connection(conn, "./migrations")
        return jsonify({
            "success": status.success,
            "migrations": [
                {"version": m.version, "name": m.name, "applied": m.applied}
                for m in status.migrations
            ]
        })
    finally:
        conn.close()

@app.route("/api/migrations/run", methods=["POST"])
def migration_run():
    conn = psycopg2.connect(DB_URI)
    try:
        result = run_on_connection(conn, "./migrations")
        return jsonify({
            "success": result.success,
            "applied": result.applied,
            "error": result.error
        })
    finally:
        conn.close()

if __name__ == "__main__":
    app.run(debug=True, port=5000)
```

### FastAPI (Modern Lifespan Context Manager)

```python
from contextlib import asynccontextmanager
import asyncio
import psycopg2
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from migradb import run_on_connection, status_on_connection, Migrator

DB_CONFIG = {
    "dbname": "myapp",
    "user": "postgres",
    "password": "secretpassword",
    "host": "localhost",
    "port": 5432,
}

migrator = Migrator("./migrations")

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Run migrations during server startup
    def _migrate():
        conn = psycopg2.connect(**DB_CONFIG)
        try:
            res = run_on_connection(conn, "./migrations")
            if not res.success:
                raise RuntimeError(f"Migration failed: {res.error}")
            print(f"Migrations applied on startup: {res.applied}")
        finally:
            conn.close()

    await asyncio.to_thread(_migrate)
    yield

app = FastAPI(lifespan=lifespan)

@app.get("/migrations/status")
async def get_status():
    def _status():
        conn = psycopg2.connect(**DB_CONFIG)
        try:
            return status_on_connection(conn, "./migrations")
        finally:
            conn.close()

    return await asyncio.to_thread(_status)

@app.post("/migrations/run")
async def run_migrations():
    def _run():
        conn = psycopg2.connect(**DB_CONFIG)
        try:
            return run_on_connection(conn, "./migrations")
        finally:
            conn.close()

    return await asyncio.to_thread(_run)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Django Management Command

```python
# myapp/management/commands/migrate_sql.py
import psycopg2
from django.conf import settings
from django.core.management.base import BaseCommand
from migradb import run_on_connection, status_on_connection

class Command(BaseCommand):
    help = "Run MigraDB SQL migrations"

    def add_arguments(self, parser):
        parser.add_argument("--dir", default="./migrations")
        parser.add_argument("--status", action="store_true")

    def handle(self, *args, **options):
        db_conf = settings.DATABASES["default"]
        conn = psycopg2.connect(
            dbname=db_conf["NAME"],
            user=db_conf["USER"],
            password=db_conf["PASSWORD"],
            host=db_conf["HOST"],
            port=db_conf["PORT"],
        )

        try:
            if options["status"]:
                status = status_on_connection(conn, options["dir"])
                for s in status.migrations:
                    status_str = "APPLIED" if s.applied else "PENDING"
                    self.stdout.write(f"[{status_str}] {s.version} - {s.name}")
            else:
                result = run_on_connection(conn, options["dir"])
                if result.success:
                    self.stdout.write(self.style.SUCCESS(f"Applied {result.applied} migrations"))
                else:
                    self.stderr.write(self.style.ERROR(result.error))
        finally:
            conn.close()
```

### SQLAlchemy Integration

```python
from sqlalchemy import create_engine, text
from migradb import Migrator

class SQLAlchemyMigration:
    def __init__(self, db_url: str, migrations_dir: str):
        self.engine = create_engine(db_url)
        self.migrator = Migrator(migrations_dir)

    def run(self):
        # Get pending migrations
        status = self.migrator.status()
        pending = [s for s in status.migrations if not s.applied]

        for migration in pending:
            # Read and execute SQL
            with open(f"{self.migrations_dir}/{migration.version}_{migration.name}.sql") as f:
                sql = f.read()

            with self.engine.begin() as conn:
                for statement in sql.split(';'):
                    if statement.strip():
                        conn.execute(text(statement))

        return len(pending)
```

## Async Support

The library is synchronous by default. For async applications, use `asyncio.to_thread`:

```python
import asyncio
from migradb import Migrator

async def run_migrations():
    m = Migrator("./migrations")

    # Run in thread pool
    result = await asyncio.to_thread(m.run)

    if result.success:
        print(f"Applied {result.applied} migrations")
    else:
        print(result.error)

asyncio.run(run_migrations())
```

### Async FastAPI Example

```python
from fastapi import FastAPI
from migradb import Migrator
import asyncio

app = FastAPI()
m = Migrator("./migrations")

@app.on_event("startup")
async def startup():
    result = await asyncio.to_thread(m.run)
    if result.success:
        print(f"Applied {result.applied} migrations")

@app.get("/migrations/status")
async def get_status():
    return await asyncio.to_thread(m.status)

@app.post("/migrations/run")
async def run_migrations():
    return await asyncio.to_thread(m.run)
```

## Generated SQLAlchemy Models

When generating models from DrawDB with `--target python` or `targetLang: "python"`, MigrDB generates complete SQLAlchemy 2.0 declarative models in `models.py`.

### Example Generated Entity

```python
# Code generated by MigraDB. DO NOT EDIT.

from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy import ForeignKey, String, Boolean, DateTime
import datetime
import uuid

class Base(DeclarativeBase):
    pass

class Company(Base):
    __tablename__ = "m_companies"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    name: Mapped[str] = mapped_column(String(200), nullable=False)

    # 1-to-many relationship navigation
    branches: Mapped[list["Branch"]] = relationship(back_populates="company")

class Branch(Base):
    __tablename__ = "m_branches"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    company_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("m_companies.id"), nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    code: Mapped[str] = mapped_column(String(50), nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    created_at: Mapped[datetime.datetime] = mapped_column(DateTime, default=datetime.datetime.utcnow)

    # Many-to-1 relationship navigation
    company: Mapped["Company"] = relationship(back_populates="branches")
```

