# Python API Reference

Complete API reference for the MigrDB Python package.

## Package: `migradb`

```python
from migradb import (
    Migrator,
    sync_database,
    detect_dialect,
    run_on_connection,
    status_on_connection,
    generate_models,
)
```

## Functions

### `sync_database(conn, ...)`

Synchronizes a live database connection against a DrawDB schema file with dynamic multi-dialect translation and snapshot tracking.

```python
def sync_database(
    conn: Any,
    schema: str = "./schema/drawdb.json",
    migrations_dir: str = "./migrations",
    schema_dir: str = "./schema",
    dialect: Optional[str] = None,
    save_migration_file: bool = True,
    migration_name: str = "sync_drawdb",
    force_full: bool = False,
) -> ResultDict: ...
```

- **Features**:
  - Auto-creates `./schema` and `./migrations` folders if missing.
  - Generates dialect-accurate DDL for PostgreSQL (default), MySQL, and SQLite.
  - Zero migration regeneration required when switching database engines midway.
- **Returns**: `ResultDict` containing `success`, `is_empty`, `applied` statements count, `diff_summary`, `migration_path`, `statements`, and optional `error`.

### `detect_dialect(conn)`

Infers the database dialect (`'postgres'`, `'mysql'`, or `'sqlite'`) from the DB-API 2.0 connection.

```python
def detect_dialect(conn: Any) -> str: ...
```

### `run_on_connection(conn, migrations_dir)`


Executes all pending migrations directly against a live Python DB-API 2.0 connection within an atomic transaction.

```python
def run_on_connection(conn: Any, migrations_dir: str) -> RunResult: ...
```

- **Supported Connections**: Any standard Python DB-API 2.0 connection (`sqlite3.Connection`, `psycopg2.extensions.connection`, `pymysql.connections.Connection`).
- **Returns**: `RunResult` containing `success`, `applied` count, `migrations`, and optional `error`.

### `status_on_connection(conn, migrations_dir)`

Inspects migration status directly against a live database connection.

```python
def status_on_connection(conn: Any, migrations_dir: str) -> StatusResult: ...
```

- **Returns**: `StatusResult` containing `success` and list of `MigrationStatus`.

### `generate_models(schema_path, target_lang, output_dir, pkg_or_namespace=None)`

Standalone function to generate models from a DrawDB diagram export file.

```python
def generate_models(
    schema_path: str,
    target_lang: str,
    output_dir: str,
    pkg_or_namespace: Optional[str] = None
) -> GenerateResult: ...
```

- **Parameters**:
  - `schema_path`: Path to the DrawDB JSON export file
  - `target_lang`: Target language (`"python"`, `"py"`, `"node"`, `"go"`, `"rust"`, `"sql"`)
  - `output_dir`: Output file or directory path
  - `pkg_or_namespace` (optional): Package name or namespace
- **Returns**: `GenerateResult` containing `success`, list of `files`, and optional `error`.

---

## Classes

### `Migrator`

The main class for managing database migrations.

```python
class Migrator:
    def __init__(self, migrations_dir: str) -> None: ...
    def run(self) -> RunResult: ...
    def status(self) -> StatusResult: ...
    def create(self, name: str, content: str) -> CreateResult: ...
    def remove_pending(self) -> RemoveResult: ...
    def generate_models(
        self,
        schema_path: str,
        target_lang: str,
        output_dir: str,
        pkg_or_namespace: Optional[str] = None
    ) -> GenerateResult: ...
```

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `__init__(self, migrations_dir: str)` | Creates instance with migrations directory |
| `run` | `run(self) -> RunResult` | Applies all pending migrations |
| `status` | `status(self) -> StatusResult` | Returns migration status |
| `create` | `create(self, name: str, content: str) -> CreateResult` | Creates a new migration file |
| `remove_pending` | `remove_pending(self) -> RemoveResult` | Removes last pending migration |
| `generate_models` | `generate_models(self, schema_path, target_lang, output_dir, pkg=None) -> GenerateResult` | Generates models from DrawDB JSON export |

---

### `Migration`

Represents a single migration.

```python
class Migration:
    version: str
    name: str
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| version | `str` | 14-digit timestamp version (e.g., "20260901000000") |
| name | `str` | Migration slug (e.g., "initial_schema") |

---

### `MigrationStatus`

Represents the status of a migration.

```python
class MigrationStatus:
    version: str
    name: str
    applied: bool
    applied_at: Optional[str]
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| version | `str` | 14-digit timestamp version |
| name | `str` | Migration slug |
| applied | `bool` | Whether the migration has been applied |
| applied_at | `Optional[str]` | ISO 8601 timestamp when applied |

---

### `RunResult`

Result from running migrations.

```python
class RunResult:
    success: bool
    applied: int
    migrations: List[Migration]
    error: Optional[str]
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| success | `bool` | Whether the operation succeeded |
| applied | `int` | Number of newly applied migrations |
| migrations | `List[Migration]` | List of applied migrations |
| error | `Optional[str]` | Error message if failed |

---

### `StatusResult`

Result from checking migration status.

```python
class StatusResult:
    success: bool
    migrations: List[MigrationStatus]
    error: Optional[str]
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| success | `bool` | Whether the operation succeeded |
| migrations | `List[MigrationStatus]` | List of all migrations with status |
| error | `Optional[str]` | Error message if failed |

---

### `CreateResult`

Result from creating a migration.

```python
class CreateResult:
    success: bool
    path: str
    error: Optional[str]
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| success | `bool` | Whether the operation succeeded |
| path | `str` | Path to the created migration file |
| error | `Optional[str]` | Error message if failed |

---

### `RemoveResult`

Result from removing a pending migration.

```python
class RemoveResult:
    success: bool
    removed: Optional[str]
    message: Optional[str]
    error: Optional[str]
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| success | `bool` | Whether the operation succeeded |
| removed | `Optional[str]` | Path of removed file |
| message | `Optional[str]` | Informational message |
| error | `Optional[str]` | Error message if failed |

---

### `GenerateResult`

Result from generating models from DrawDB schema.

```python
class GenerateResult:
    success: bool
    files: List[str]
    error: Optional[str]
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| success | `bool` | Whether code generation succeeded |
| files | `List[str]` | List of generated file paths |
| error | `Optional[str]` | Error message if failed |

## Usage Examples

### Constructor

```python
m = Migrator("./migrations")
```

### Run Migrations

```python
result = m.run()

if result.success:
    print(f"Applied {result.applied} migrations")
    for mig in result.migrations:
        print(f"  - {mig.version}: {mig.name}")
else:
    print(result.error)
```

### Check Status

```python
status = m.status()

if status.success:
    for s in status.migrations:
        status_str = "APPLIED" if s.applied else "PENDING"
        print(f"[{status_str}] {s.version} - {s.name}")
```

### Create Migration

```python
result = m.create("add products table", """
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL,
        price DECIMAL(10,2) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
""")

if result.success:
    print(f"Created: {result.path}")
```

### Remove Pending Migration

```python
result = m.remove_pending()

if result.success:
    if result.removed:
        print(f"Removed: {result.removed}")
    else:
        print(result.message)
```

### Direct Database Migration (`run_on_connection`)

Execute migrations against real DB-API 2.0 connections (`sqlite3`, `psycopg2`, `pymysql`) in an atomic transaction:

```python
import psycopg2
from migradb import run_on_connection, status_on_connection

conn = psycopg2.connect("dbname=myapp user=postgres password=secret host=localhost")

# Run migrations
result = run_on_connection(conn, "./migrations")
if result.success:
    print(f"Applied {result.applied} migrations")
else:
    print(f"Failed: {result.error}")

# Check status
status = status_on_connection(conn, "./migrations")
for s in status.migrations:
    print(f"[{'APPLIED' if s.applied else 'PENDING'}] {s.version} - {s.name}")

conn.close()
```

### Model Generation (`generate_models`)

Generate SQLAlchemy 2.0 declarative models from DrawDB JSON export:

```python
from migradb import generate_models

result = generate_models("schema/drawdb.json", "python", "./app/models.py")
if result.success:
    print(f"Generated {len(result.files)} files: {result.files}")
else:
    print(f"Generation error: {result.error}")
```

## Error Handling

All methods return result objects with a `success` boolean. Check this first before accessing other properties.

```python
result = m.run()

if not result.success:
    # Handle error
    print(result.error)
    return

# Handle success
print(f"Applied {result.applied} migrations")
```

## Type Hints

The library supports Python type hints:

```python
from migradb import (
    Migrator,
    RunResult,
    StatusResult,
    CreateResult,
    RemoveResult,
    GenerateResult,
    run_on_connection,
    status_on_connection,
    generate_models,
)

m: Migrator = Migrator("./migrations")
result: RunResult = m.run()

if result.success:
    applied: int = result.applied
    migrations: list = result.migrations
```

## Async Usage

For async applications, use `asyncio.to_thread`:

```python
import asyncio
from migradb import Migrator

async def run_migrations():
    m = Migrator("./migrations")
    result = await asyncio.to_thread(m.run)
    return result

## Generated SQLAlchemy 2.0 Models

When target language is set to Python (`targetLang: "python"`), MigrDB's model engine outputs modern, idiomatic SQLAlchemy 2.0 declarative models in `models.py`:

- **Declarative Base**: Inherits from a shared `DeclarativeBase`.
- **Typed Attributes**: Uses modern `Mapped[...]` annotations and `mapped_column(...)`.
- **Relationships**: Automatically binds foreign keys using `relationship(...)` with back-populates.

```python
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy import ForeignKey, String, Boolean, DateTime
import datetime
import uuid

class Base(DeclarativeBase):
    pass

class Branch(Base):
    __tablename__ = "m_branches"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    company_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("m_companies.id"), nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    code: Mapped[str] = mapped_column(String(50), nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)
    created_at: Mapped[datetime.datetime] = mapped_column(DateTime, default=datetime.datetime.utcnow)

    # Relationships
    company: Mapped["Company"] = relationship(back_populates="branches")
```

## Platform Support

Pre-built wheels are available for:

| Platform | Python Versions | Architectures |
|----------|-----------------|---------------|
| Linux | 3.8 - 3.12 | x86_64, aarch64 |
| macOS | 3.8 - 3.12 | x86_64, arm64 |
| Windows | 3.8 - 3.12 | x86, AMD64 |

## Performance

The Rust core provides:

- **Fast parsing**: Migration files parsed in microseconds
- **Low memory**: Minimal memory footprint
- **GIL-friendly**: Releases GIL during operations for better concurrency
