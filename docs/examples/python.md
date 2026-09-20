# Python Examples

Practical, production-ready examples for using MigrDB in Python projects.

## Table of Contents

- [Real Database Connections](#real-database-connections)
  - [PostgreSQL (`psycopg2`)](#postgresql-psycopg2)
  - [SQLite (`sqlite3`)](#sqlite-sqlite3)
  - [MySQL (`pymysql`)](#mysql-pymysql)
- [FastAPI Integration (Lifespan)](#fastapi-integration-lifespan)
- [Flask Integration](#flask-integration)
- [DrawDB Model Generation (SQLAlchemy 2.0)](#drawdb-model-generation-sqlalchemy-20)
- [Using the CLI](#using-the-cli)
- [Multi-Tenant Migrations](#multi-tenant-migrations)
- [Dynamic Migration Creator](#dynamic-migration-creator)

---

## Real Database Connections

MigrDB provides `run_on_connection` and `status_on_connection` which operate directly on standard Python DB-API 2.0 connections, ensuring transactions and recording history in `schema_migrations`.

### PostgreSQL (`psycopg2`)

```bash
pip install migradb psycopg2-binary
```

```python
import psycopg2
from migradb import run_on_connection, status_on_connection

conn = psycopg2.connect(
    dbname="myapp",
    user="postgres",
    password="secretpassword",
    host="localhost",
    port=5432,
)

try:
    # 1. Inspect migration status
    status = status_on_connection(conn, "./migrations")
    print("Migration Status:")
    for m in status.migrations:
        status_label = "APPLIED" if m.applied else "PENDING"
        print(f"  [{status_label}] {m.version} - {m.name}")

    # 2. Apply all pending migrations in atomic transactions
    result = run_on_connection(conn, "./migrations")
    if result.success:
        print(f"\nApplied {result.applied} migrations successfully!")
    else:
        print(f"\nMigration failed: {result.error}")
finally:
    conn.close()
```

---

### SQLite (`sqlite3`)

```python
import sqlite3
from migradb import run_on_connection, status_on_connection

conn = sqlite3.connect("app.db")

try:
    # Check status
    status = status_on_connection(conn, "./migrations")
    print(f"Total migrations: {len(status.migrations)}")

    # Run pending migrations
    result = run_on_connection(conn, "./migrations")
    if result.success:
        print(f"Applied {result.applied} SQLite migration(s)")
    else:
        print(f"Error: {result.error}")
finally:
    conn.close()
```

---

### MySQL (`pymysql`)

```bash
pip install migradb pymysql
```

```python
import pymysql
from migradb import run_on_connection, status_on_connection

conn = pymysql.connect(
    host="localhost",
    user="root",
    password="secretpassword",
    database="myapp",
)

try:
    result = run_on_connection(conn, "./migrations")
    if result.success:
        print(f"Applied {result.applied} MySQL migrations")
    else:
        print(f"Migration error: {result.error}")
finally:
    conn.close()
```

---

## FastAPI Integration (Lifespan)

Use FastAPI's modern `lifespan` context manager to run pending migrations automatically when the application starts:

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
    # Run migrations during server startup in a worker thread
    def _run_migrations():
        conn = psycopg2.connect(**DB_CONFIG)
        try:
            result = run_on_connection(conn, "./migrations")
            if not result.success:
                raise RuntimeError(f"Database migration failed: {result.error}")
            print(f"Migrations applied: {result.applied}")
        finally:
            conn.close()

    await asyncio.to_thread(_run_migrations)
    yield
    # Cleanup on shutdown if needed

app = FastAPI(lifespan=lifespan)

class CreateMigrationRequest(BaseModel):
    name: str
    content: str = "-- Add SQL here\n"

@app.get("/api/migrations/status")
async def get_status():
    def _get():
        conn = psycopg2.connect(**DB_CONFIG)
        try:
            status = status_on_connection(conn, "./migrations")
            return [
                {
                    "version": m.version,
                    "name": m.name,
                    "applied": m.applied,
                    "applied_at": m.applied_at,
                }
                for m in status.migrations
            ]
        finally:
            conn.close()

    return await asyncio.to_thread(_get)

@app.post("/api/migrations/create")
def create_migration(req: CreateMigrationRequest):
    res = migrator.create(req.name, req.content)
    if not res.success:
        raise HTTPException(status_code=400, detail=res.error)
    return {"path": res.path}
```

---

## Flask Integration

Run migrations on application startup within an application context:

```python
import os
import psycopg2
from flask import Flask, jsonify, request
from migradb import run_on_connection, status_on_connection, Migrator

app = Flask(__name__)
migrator = Migrator("./migrations")

DATABASE_URL = os.getenv("DATABASE_URL", "dbname=myapp user=postgres password=secret host=localhost")

def apply_migrations():
    conn = psycopg2.connect(DATABASE_URL)
    try:
        res = run_on_connection(conn, "./migrations")
        if not res.success:
            raise RuntimeError(res.error)
        print(f"Applied {res.applied} migrations")
    finally:
        conn.close()

# Run migrations at startup
with app.app_context():
    apply_migrations()

@app.route("/api/migrations/status")
def migration_status():
    conn = psycopg2.connect(DATABASE_URL)
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

@app.route("/api/migrations/create", methods=["POST"])
def create_migration():
    data = request.get_json() or {}
    name = data.get("name")
    if not name:
        return jsonify({"error": "Migration name is required"}), 400
    res = migrator.create(name, data.get("content", "-- SQL\n"))
    return jsonify({"path": res.path})

if __name__ == "__main__":
    app.run(port=5000, debug=True)
```

---

## DrawDB Model Generation (SQLAlchemy 2.0)

Export your DrawDB diagram as JSON and generate production-ready SQLAlchemy 2.0 models with typed columns and relationships:

```python
from migradb import generate_models

# Generate models into app/models.py
result = generate_models(
    schema_path="schema/drawdb.json",
    target_lang="python",
    output_dir="./app/models.py"
)

if result.success:
    print(f"Successfully generated models: {result.files}")
else:
    print(f"Failed: {result.error}")
```

The generated `models.py` uses modern SQLAlchemy 2.0 syntax:

```python
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
    branches: Mapped[list["Branch"]] = relationship(back_populates="company")

class Branch(Base):
    __tablename__ = "m_branches"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    company_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("m_companies.id"), nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    company: Mapped["Company"] = relationship(back_populates="branches")
```

---

## Using the CLI

You can use the standalone compiled `migradb` binary directly:

```bash
# Initialize migrations folder
migradb init --dir ./migrations

# Create a new timestamped migration
migradb create add_users_table --dir ./migrations

# Inspect migration file status
migradb status --dir ./migrations

# Generate SQLAlchemy models from DrawDB JSON diagram
migradb generate --schema drawdb.json --lang python --out ./app/models.py
```

---

## Multi-Tenant Migrations

Execute migrations across multiple tenant databases sequentially:

```python
import psycopg2
from migradb import run_on_connection

tenants = [
    {"name": "tenant_1", "dsn": "dbname=tenant1 user=postgres password=secret host=localhost"},
    {"name": "tenant_2", "dsn": "dbname=tenant2 user=postgres password=secret host=localhost"},
]

for tenant in tenants:
    print(f"Migrating {tenant['name']}...")
    conn = psycopg2.connect(tenant["dsn"])
    try:
        res = run_on_connection(conn, "./migrations")
        if res.success:
            print(f"  [OK] Applied {res.applied} migrations")
        else:
            print(f"  [FAIL] {res.error}")
    finally:
        conn.close()
```

---

## Dynamic Migration Creator

Create migration files dynamically from Python scripts:

```python
from migradb import Migrator

m = Migrator("./migrations")

def create_table_migration(table_name: str, columns: list[tuple[str, str]]):
    col_lines = ",\n    ".join(f"{name} {col_type}" for name, col_type in columns)
    sql = f"CREATE TABLE IF NOT EXISTS {table_name} (\n    {col_lines}\n);\n"
    return m.create(f"create_{table_name}_table", sql)

result = create_table_migration("orders", [
    ("id", "SERIAL PRIMARY KEY"),
    ("user_id", "INTEGER NOT NULL"),
    ("total_amount", "DECIMAL(10,2) NOT NULL DEFAULT 0.00"),
    ("created_at", "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP"),
])

if result.success:
    print(f"Created migration: {result.path}")
```
