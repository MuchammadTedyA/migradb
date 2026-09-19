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

```python
from migradb import Migrator

# Create migrator instance
m = Migrator("./migrations")

# Run all pending migrations
result = m.run()

if result.success:
    print(f"Applied {result.applied} migrations")
else:
    print(result.error)
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

### Flask

```python
from flask import Flask, jsonify
from migradb import Migrator

app = Flask(__name__)
m = Migrator("./migrations")

# Run migrations on startup
with app.app_context():
    result = m.run()
    if result.success:
        print(f"Applied {result.applied} migrations")
    else:
        print(f"Migration failed: {result.error}")

@app.route("/api/migrations/status")
def migration_status():
    return jsonify(m.status().__dict__)

@app.route("/api/migrations/run", methods=["POST"])
def migration_run():
    return jsonify(m.run().__dict__)

if __name__ == "__main__":
    app.run(debug=True)
```

### FastAPI

```python
from fastapi import FastAPI, HTTPException
from migradb import Migrator, StatusResult, RunResult

app = FastAPI()
m = Migrator("./migrations")

@app.on_event("startup")
async def startup_event():
    result = m.run()
    if not result.success:
        raise RuntimeError(result.error)
    print(f"Applied {result.applied} migrations")

@app.get("/migrations/status", response_model=dict)
async def get_status():
    return m.status().__dict__

@app.post("/migrations/run", response_model=dict)
async def run_migrations():
    return m.run().__dict__

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Django

```python
# management/commands/migrate_custom.py
from django.core.management.base import BaseCommand
from migradb import Migrator

class Command(BaseCommand):
    help = 'Run custom migrations'

    def add_arguments(self, parser):
        parser.add_argument('--dir', default='./migrations')
        parser.add_argument('--status', action='store_true')

    def handle(self, *args, **options):
        m = Migrator(options['dir'])

        if options['status']:
            status = m.status()
            for s in status.migrations:
                status_str = 'APPLIED' if s.applied else 'PENDING'
                self.stdout.write(f'[{status_str}] {s.version} - {s.name}')
        else:
            result = m.run()
            if result.success:
                self.stdout.write(self.style.SUCCESS(
                    f'Applied {result.applied} migrations'
                ))
            else:
                self.stderr.write(self.style.ERROR(result.error))
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

