# Python API Reference

Complete API reference for the MigrDB Python package.

## Package: `migradb`

```python
from migradb import Migrator
```

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
```

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `__init__` | `__init__(self, migrations_dir: str)` | Creates instance with migrations directory |
| `run` | `run(self) -> RunResult` | Applies all pending migrations |
| `status` | `status(self) -> StatusResult` | Returns migration status |
| `create` | `create(self, name: str, content: str) -> CreateResult` | Creates a new migration file |
| `remove_pending` | `remove_pending(self) -> RemoveResult` | Removes last pending migration |

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
from migradb import Migrator, RunResult, StatusResult

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
