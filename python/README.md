# migradb

A fast, lightweight SQL database migration library with a Rust core engine for Python. Works with any standard DB-API 2.0 database connection (PostgreSQL, MySQL, SQLite, etc.).

## Features

- ⚡ **Fast & Lightweight**: Native Rust parsing and validation engine compiled with PyO3.
- 🔄 **Universal Schema Sync (Approach 1)**: Sync live databases directly from DrawDB (`drawdb.json`) with zero migration regeneration on database switch.
- 🎯 **Multi-Dialect Support**: PostgreSQL by default, with complete native support for MySQL and SQLite.
- 📁 **Auto Directory Conventions**: Auto-creates `./schema` and `./migrations` if missing; allows custom paths.
- 🔄 **Universal DB-API 2.0 Support**: Works seamlessly with `sqlite3`, `psycopg2` / `psycopg3`, `pymysql`, `mysqlclient`, etc.
- 🛡️ **Transactional Safety**: Automatically executes each migration within its own atomic transaction.
- 📜 **Plain SQL Files**: Standard `.sql` files with timestamp-based ordering (`YYYYMMDDHHmmss_name.sql`).
- 🎨 **DrawDB Support**: Auto-generate SQL migration scripts and SQLAlchemy 2.0 / Pydantic models from `drawdb.json`.

---

## 1. Installation

```bash
pip install migradb
```

Plus your preferred database driver:

```bash
# For PostgreSQL (default dialect):
pip install psycopg2-binary
# For MySQL:
pip install pymysql
# SQLite is built directly into Python standard library!
```

---

## 2. Universal Schema Sync (`sync_database`) - Recommended

Design your database schema visually in DrawDB, export the JSON, and let MigraDB synchronize your database on application start.

### Why Approach 1?
- **Zero Migration Regeneration**: If you change your database midway (e.g., SQLite during prototyping -> PostgreSQL in production, or MySQL -> PostgreSQL), **you do not need to rewrite or regenerate any migrations**. Simply connect to the new database, and MigraDB generates the correct DDL syntax on the fly!
- **Default Directory Conventions**: Automatically creates `./schema` and `./migrations` in the root project if they do not exist. If they already exist, files are placed directly inside.
- **Dialect Defaults**: Defaults to **PostgreSQL**. MySQL and SQLite are fully supported.
- **Audit Logging**: Automatically writes timestamped audit migrations (`migrations/YYYYMMDDHHmmss_sync_drawdb.sql`) and maintains snapshots (`schema/drawdb_snapshot.json`).

### A. With PostgreSQL (`psycopg2`)
```python
import psycopg2
from migradb import sync_database

conn = psycopg2.connect("dbname=mydb user=postgres password=secret host=localhost")

# Synchronize against DrawDB schema
res = sync_database(
    conn=conn,
    schema="./schema/drawdb.json",    # Default schema path
    migrations_dir="./migrations",     # Auto-created if missing
    schema_dir="./schema",             # Auto-created if missing
    save_migration_file=True,          # Generates audit .sql migration
    migration_name="init_schema",      # Migration slug
)

if res.is_empty:
    print("✅ Schema is already up to date.")
elif res.success:
    print(f"✅ Applied {res.applied} statements! Audit file: {res.migration_path}")
else:
    print(f"❌ Sync failed: {res.error}")

conn.close()
```

### B. With SQLite (Built-in `sqlite3`)
```python
import sqlite3
from migradb import sync_database

conn = sqlite3.connect("app.db")
res = sync_database(conn, schema="./schema/drawdb.json")
print(f"Applied {res.applied} statements to SQLite!")
conn.close()
```

### C. With MySQL (`pymysql`)
```python
import pymysql
from migradb import sync_database

conn = pymysql.connect(host="localhost", user="root", password="", database="mydb")
res = sync_database(conn, schema="./schema/drawdb.json", dialect="mysql")
print(f"Applied {res.applied} statements to MySQL!")
conn.close()
```

---

## 3. Applying Manual Migration Files (`run_on_connection`)

Create a `migrations/` directory in your project:

```text
my_project/
├── migrations/
│   ├── 20260901000000_create_users.sql
│   └── 20260901000100_add_roles.sql
├── main.py
└── requirements.txt
```

**`migrations/20260901000000_create_users.sql`:**
```sql
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 3. Applying Migrations in Python

Pass any standard database connection to `run_on_connection`:

### A. With PostgreSQL (`psycopg2`)

```python
import psycopg2
from migradb import run_on_connection, status_on_connection

conn = psycopg2.connect("dbname=mydb user=postgres password=secret host=localhost")

# 1. Check status
status = status_on_connection(conn, "./migrations")
print("Status:", status["migrations"])

# 2. Run pending migrations
result = run_on_connection(conn, "./migrations")
if result["success"]:
    print(f"✅ Applied {result['applied']} migrations!")
else:
    print(f"❌ Error: {result['error']}")

conn.close()
```

---

### B. With SQLite (Built-in `sqlite3`)

```python
import sqlite3
from migradb import run_on_connection

conn = sqlite3.connect("app.db")
result = run_on_connection(conn, "./migrations")
print(f"Applied {result['applied']} migrations!")
conn.close()
```

---

### C. With MySQL (`pymysql`)

```python
import pymysql
from migradb import run_on_connection

conn = pymysql.connect(host="localhost", user="root", password="", database="mydb")
result = run_on_connection(conn, "./migrations")
print(f"Applied {result['applied']} migrations!")
conn.close()
```

---

## 4. Generate Migrations & Models from DrawDB (`drawdb.json`)

You can generate production-ready SQL migration files and Python dataclass models directly from your visual diagrams!

```python
from migradb import generate_models

# 1. Generate PostgreSQL migration SQL files directly from drawdb.json:
sql_result = generate_models(
    schema_path="schema/drawdb.json",
    target_lang="sql",
    output_dir="./migrations",
    pkg_or_namespace="postgres", # 'postgres', 'mysql', or 'sqlite'
)
if sql_result.success:
    print("Generated SQL migration files:", sql_result.files)

# 2. Generate Python dataclass models from the same diagram:
py_result = generate_models(
    schema_path="schema/drawdb.json",
    target_lang="python",
    output_dir="./models",
)
if py_result.success:
    print("Generated Python models:", py_result.files)
```

---

## 5. FastAPI / Lifespan Integration

Synchronize schema automatically when your web application starts up:

```python
from contextlib import asynccontextmanager
from fastapi import FastAPI
import psycopg2
from migradb import sync_database

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Automatically sync database against DrawDB visual schema at startup
    conn = psycopg2.connect("dbname=mydb user=postgres password=secret host=localhost")
    sync_database(conn, schema="./schema/drawdb.json")
    conn.close()
    yield

app = FastAPI(lifespan=lifespan)
```

---

## License

MIT © [MuchammadTedyA](https://github.com/MuchammadTedyA)
