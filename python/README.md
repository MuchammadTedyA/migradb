# migradb

A high-performance SQL database migration library with a Rust core engine, providing Python bindings via PyO3.

## Installation

```bash
pip install migradb
```

## Quick Start

```python
from migradb import Migrator

m = Migrator("./migrations")

# Check migration status
status = m.status()
if status.success:
    for s in status.migrations:
        status_label = "APPLIED" if s.applied else "PENDING"
        print(f"[{status_label}] {s.version} - {s.name}")

# Run pending migrations
result = m.run()
if result.success:
    print(f"Applied {result.applied} migrations")
else:
    print(f"Error: {result.error}")
```

## Async Usage (FastAPI, Asyncio)

```python
import asyncio
from migradb import Migrator

async def run_migrations():
    m = Migrator("./migrations")
    result = await asyncio.to_thread(m.run)
    return result

asyncio.run(run_migrations())
```

## Features

- **Timestamp-based versioning**: `YYYYMMDDHHmmss_slug.sql` format
- **Transactional safety**: Each migration executes within an isolated transaction
- **Immutable history**: Never edit old migrations; roll-forward design
- **Fast Rust Core**: Parsing and version checks powered by native Rust

## License

MIT

