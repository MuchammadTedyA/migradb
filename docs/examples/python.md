# Python Examples

Practical examples for using MigrDB in Python projects.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Flask Integration](#flask-integration)
- [FastAPI Integration](#fastapi-integration)
- [Django Integration](#django-integration)
- [CLI Tool](#cli-tool)
- [Multi-Database](#multi-database)
- [Migration Generator](#migration-generator)

## Basic Usage

```python
from migradb import Migrator

m = Migrator("./migrations")

# Check status
status = m.status()
if status.success:
    print("Migration Status:")
    for s in status.migrations:
        status_str = "APPLIED" if s.applied else "PENDING"
        print(f"  [{status_str}] {s.version} - {s.name}")

# Run migrations
result = m.run()
if result.success:
    print(f"\nApplied {result.applied} migrations")
```

## Flask Integration

```python
from flask import Flask, jsonify, request
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

@app.route("/api/migrations/create", methods=["POST"])
def migration_create():
    data = request.json
    result = m.create(data["name"], data["content"])
    return jsonify(result.__dict__)

if __name__ == "__main__":
    app.run(debug=True)
```

## FastAPI Integration

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from migradb import Migrator

app = FastAPI()
m = Migrator("./migrations")

class CreateMigrationRequest(BaseModel):
    name: str
    content: str

@app.on_event("startup")
async def startup_event():
    result = m.run()
    if not result.success:
        raise RuntimeError(result.error)
    print(f"Applied {result.applied} migrations")

@app.get("/migrations/status")
async def get_status():
    return m.status().__dict__

@app.post("/migrations/run")
async def run_migrations():
    return m.run().__dict__

@app.post("/migrations/create")
async def create_migration(request: CreateMigrationRequest):
    result = m.create(request.name, request.content)
    if not result.success:
        raise HTTPException(status_code=400, detail=result.error)
    return result.__dict__

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

## Django Integration

### Management Command

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
            if status.success:
                for s in status.migrations:
                    status_str = 'APPLIED' if s.applied else 'PENDING'
                    self.stdout.write(f'[{status_str}] {s.version} - {s.name}')
            else:
                self.stderr.write(self.style.ERROR(status.error))
        else:
            result = m.run()
            if result.success:
                self.stdout.write(self.style.SUCCESS(
                    f'Applied {result.applied} migrations'
                ))
            else:
                self.stderr.write(self.style.ERROR(result.error))
```

### Django Settings

```python
# settings.py
import os
from migradb import Migrator

MIGRATIONS_DIR = os.path.join(BASE_DIR, 'migrations')
migrator = Migrator(MIGRATIONS_DIR)

# Run migrations on startup (in AppConfig)
# apps.py
from django.apps import AppConfig

class MyAppConfig(AppConfig):
    name = 'myapp'

    def ready(self):
        from django.conf import settings
        result = settings.migrator.run()
        if result.success:
            print(f"Applied {result.applied} migrations")
```

## CLI Tool

```python
#!/usr/bin/env python3
import sys
import os
from migradb import Migrator

def print_usage():
    print("Usage: migrate <command> [options]")
    print("")
    print("Commands:")
    print("  run      Apply pending migrations")
    print("  status   Show migration status")
    print("  create   Create new migration file")
    print("  remove   Remove last pending migration")
    print("")
    print("Options:")
    print("  --dir    Migrations directory (default: ./migrations)")

def handle_run(migrations_dir):
    m = Migrator(migrations_dir)
    result = m.run()

    if not result.success:
        print(f"Error: {result.error}", file=sys.stderr)
        sys.exit(1)

    if result.applied == 0:
        print("No pending migrations")
    else:
        print(f"Applied {result.applied} migrations:")
        for mig in result.migrations:
            print(f"  - {mig.version}: {mig.name}")

def handle_status(migrations_dir):
    m = Migrator(migrations_dir)
    status = m.status()

    if not status.success:
        print(f"Error: {status.error}", file=sys.stderr)
        sys.exit(1)

    for s in status.migrations:
        status_str = "APPLIED" if s.applied else "PENDING"
        print(f"[{status_str}] {s.version} - {s.name}")

def handle_create(migrations_dir, name):
    if not name:
        print("Error: Migration name required", file=sys.stderr)
        print("Usage: migrate create <name>", file=sys.stderr)
        sys.exit(1)

    m = Migrator(migrations_dir)
    result = m.create(name, "-- Add your SQL here\n")

    if not result.success:
        print(f"Error: {result.error}", file=sys.stderr)
        sys.exit(1)

    print(f"Created: {result.path}")

def handle_remove(migrations_dir):
    m = Migrator(migrations_dir)
    result = m.remove_pending()

    if not result.success:
        print(f"Error: {result.error}", file=sys.stderr)
        sys.exit(1)

    if result.removed:
        print(f"Removed: {result.removed}")
    else:
        print(result.message)

def main():
    args = sys.argv[1:]

    if not args:
        print_usage()
        return

    migrations_dir = "./migrations"
    command = args[0]

    # Parse options
    i = 1
    while i < len(args):
        if args[i] == "--dir" and i + 1 < len(args):
            migrations_dir = args[i + 1]
            i += 2
        else:
            i += 1

    if command == "run":
        handle_run(migrations_dir)
    elif command == "status":
        handle_status(migrations_dir)
    elif command == "create":
        name = args[1] if len(args) > 1 else None
        handle_create(migrations_dir, name)
    elif command == "remove":
        handle_remove(migrations_dir)
    else:
        print(f"Unknown command: {command}", file=sys.stderr)
        print_usage()
        sys.exit(1)

if __name__ == "__main__":
    main()
```

## Multi-Database

```python
from migradb import Migrator
from concurrent.futures import ThreadPoolExecutor, as_completed

databases = {
    "users": Migrator("./migrations/users"),
    "orders": Migrator("./migrations/orders"),
    "products": Migrator("./migrations/products"),
}

def run_migrations(name, m):
    result = m.run()
    return name, result

def check_status(name, m):
    status = m.status()
    return name, status

# Run all migrations
with ThreadPoolExecutor(max_workers=3) as executor:
    futures = {
        executor.submit(run_migrations, name, m): name
        for name, m in databases.items()
    }

    for future in as_completed(futures):
        name, result = future.result()
        if result.success:
            print(f"{name}: Applied {result.applied} migrations")
        else:
            print(f"{name}: Error - {result.error}")

# Check all status
with ThreadPoolExecutor(max_workers=3) as executor:
    futures = {
        executor.submit(check_status, name, m): name
        for name, m in databases.items()
    }

    for future in as_completed(futures):
        name, status = future.result()
        if status.success:
            print(f"\n{name}:")
            for s in status.migrations:
                status_str = "APPLIED" if s.applied else "PENDING"
                print(f"  [{status_str}] {s.version} - {s.name}")
```

## Migration Generator

```python
from migradb import Migrator
from typing import List, Optional

class TableColumn:
    def __init__(
        self,
        name: str,
        type: str,
        nullable: bool = True,
        default: Optional[str] = None,
        primary_key: bool = False,
        unique: bool = False,
    ):
        self.name = name
        self.type = type
        self.nullable = nullable
        self.default = default
        self.primary_key = primary_key
        self.unique = unique

class TableMigration:
    def __init__(self, migrator: Migrator):
        self.migrator = migrator

    def create_table(self, table_name: str, columns: List[TableColumn]):
        column_defs = []
        indexes = []

        for col in columns:
            def_str = f"{col.name} {col.type}"

            if not col.nullable:
                def_str += " NOT NULL"
            if col.default:
                def_str += f" DEFAULT {col.default}"
            if col.primary_key:
                def_str += " PRIMARY KEY"
            if col.unique:
                def_str += " UNIQUE"

            column_defs.append(def_str)

            if not col.primary_key:
                indexes.append(
                    f"CREATE INDEX IF NOT EXISTS idx_{table_name}_{col.name} "
                    f"ON {table_name}({col.name});"
                )

        sql = (
            f"CREATE TABLE IF NOT EXISTS {table_name} (\n"
            f"    {', '.join(column_defs)}\n"
            f");\n\n"
            f"{' '.join(indexes)}"
        )

        return self.migrator.create(f"create {table_name} table", sql)

    def add_column(self, table_name: str, column: TableColumn):
        sql = f"ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS {column.name} {column.type}"

        if not column.nullable:
            sql += " NOT NULL"
        if column.default:
            sql += f" DEFAULT {column.default}"

        return self.migrator.create(f"add {column.name} to {table_name}", sql + ";")

    def add_index(self, table_name: str, columns: List[str], unique: bool = False):
        index_type = "UNIQUE INDEX" if unique else "INDEX"
        index_name = f"idx_{table_name}_{'_'.join(columns)}"
        sql = (
            f"CREATE {index_type} IF NOT EXISTS {index_name} "
            f"ON {table_name}({', '.join(columns)});"
        )

        return self.migrator.create(f"add index to {table_name}", sql)

# Usage
m = Migrator("./migrations")
tm = TableMigration(m)

# Create users table
result = tm.create_table("m_users", [
    TableColumn("id", "UUID", primary_key=True, default="gen_random_uuid()"),
    TableColumn("username", "VARCHAR(100)", unique=True, nullable=False),
    TableColumn("email", "VARCHAR(255)", unique=True, nullable=False),
    TableColumn("password_hash", "TEXT", nullable=False),
    TableColumn("is_active", "BOOLEAN", default="true"),
    TableColumn("created_at", "TIMESTAMPTZ", default="NOW()"),
])

if result.success:
    print(f"Created: {result.path}")

# Add column
tm.add_column("m_users", TableColumn("phone", "VARCHAR(20)", nullable=True))

# Add index
tm.add_index("m_users", ["email"])
tm.add_index("m_users", ["email", "username"], unique=True)
```

## Environment-Based Configuration

```python
import os
from migradb import Migrator

class MigrationConfig:
    def __init__(self):
        self.migrations_dir = os.getenv("MIGRATIONS_DIR", "./migrations")
        self.environment = os.getenv("APP_ENV", "development")
        self.auto_migrate = os.getenv("AUTO_MIGRATE", "true").lower() == "true"

    def run_migrations(self):
        m = Migrator(self.migrations_dir)

        if self.auto_migrate and self.environment != "test":
            result = m.run()
            if result.success:
                print(f"[{self.environment}] Applied {result.applied} migrations")
            else:
                print(f"[{self.environment}] Migration failed: {result.error}")
                raise RuntimeError(result.error)

        return m

# Usage
config = MigrationConfig()
m = config.run_migrations()
```
