# migradb

A fast, lightweight SQL database migration library with a Rust core engine for Python. Works with any standard DB-API 2.0 database connection (PostgreSQL, MySQL, SQLite, etc.).

## Features

- ⚡ **Fast & Lightweight**: Native Rust parsing and validation engine compiled with PyO3.
- 🔄 **Universal DB-API 2.0 Support**: Works seamlessly with `sqlite3`, `psycopg2` / `psycopg3`, `pymysql`, `mysqlclient`, etc.
- 🛡️ **Transactional Safety**: Automatically executes each migration within its own transaction.
- 📜 **Plain SQL Files**: Standard `.sql` files with timestamp-based ordering (`YYYYMMDDHHmmss_name.sql`).
- 🎨 **DrawDB Support**: Auto-generate SQL migration scripts and Python Pydantic/dataclass models from `drawdb.json`.

---

## 1. Installation

```bash
pip install migradb
```

Plus your preferred database driver:

```bash
# For PostgreSQL:
pip install psycopg2-binary
# For MySQL:
pip install pymysql
# SQLite is built directly into Python!
```

---

## 2. Create Migration Files

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

Run migrations automatically when your web application starts up:

```python
from contextlib import asynccontextmanager
from fastapi import FastAPI
import psycopg2
from migradb import run_on_connection

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Run pending migrations before accepting HTTP requests
    conn = psycopg2.connect("dbname=mydb user=postgres password=secret host=localhost")
    run_on_connection(conn, "./migrations")
    conn.close()
    yield

app = FastAPI(lifespan=lifespan)
```

---

## License

MIT © [MuchammadTedyA](https://github.com/MuchammadTedyA)
