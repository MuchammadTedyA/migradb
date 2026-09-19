# Database Guide

Database-specific configuration and best practices for the MigrDB.

## Table of Contents

- [PostgreSQL](#postgresql)
- [MySQL](#mysql)
- [SQLite](#sqlite)
- [Connection Management](#connection-management)
- [Transaction Handling](#transaction-handling)
- [Schema Migrations Table](#schema-migrations-table)

## PostgreSQL

### Connection String Format

```
postgresql://user:password@host:port/database?sslmode=require
```

### Recommended Settings

```sql
-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Set timezone
SET timezone = 'UTC';
```

### Migration Example

```sql
-- Migration: create_users_table
CREATE TABLE IF NOT EXISTS m_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON m_users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON m_users(username);
```

### Go Integration with pgx

```go
package main

import (
    "context"
    "github.com/jackc/pgx/v5/pgxpool"
    "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    ctx := context.Background()

    config, _ := pgxpool.ParseConfig("postgres://user:pass@localhost/dbname")
    db, err := pgxpool.NewWithConfig(ctx, config)
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    // Run migrations
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    result, err := m.Run()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Applied %d migrations\n", result.Applied)
}
```

### Node.js Integration with pg

```javascript
const { Pool } = require('pg');
const { Migrator } = require('migradb');

const pool = new Pool({
    connectionString: 'postgres://user:pass@localhost/dbname'
});

const m = new Migrator('./migrations');
const result = m.run();
```

### Python Integration with asyncpg

```python
import asyncpg
from migradb import Migrator

async def main():
    conn = await asyncpg.connect('postgres://user:pass@localhost/dbname')

    m = Migrator("./migrations")
    result = m.run()

    await conn.close()
```

## MySQL

### Connection String Format

```
mysql://user:password@host:port/database
```

### Recommended Settings

```sql
-- Set charset
SET NAMES utf8mb4;
SET CHARACTER SET utf8mb4;

-- Set timezone
SET time_zone = '+00:00';
```

### Migration Example

```sql
-- Migration: create_users_table
CREATE TABLE IF NOT EXISTS m_users (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE INDEX idx_users_email ON m_users(email);
CREATE INDEX idx_users_username ON m_users(username);
```

### Notes for MySQL

- Use `CHAR(36)` for UUID instead of native UUID type
- Use `ON UPDATE CURRENT_TIMESTAMP` for auto-updating timestamps
- Specify `ENGINE=InnoDB` for transaction support
- Use `utf8mb4` charset for full Unicode support

## SQLite

### Connection String Format
```
sqlite:///path/to/database.db
```

### Recommended Settings

```sql
-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Set journal mode
PRAGMA journal_mode = WAL;
```

### Migration Example

```sql
-- Migration: create_users_table
CREATE TABLE IF NOT EXISTS m_users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_email ON m_users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON m_users(username);
```

### Notes for SQLite

- Use `TEXT` for UUID (SQLite has no UUID type)
- Use `INTEGER` for boolean (0/1)
- Use `datetime('now')` for timestamps
- Enable `foreign_keys` pragma for FK support
- Use `WAL` journal mode for better concurrency

## Connection Management

### Connection Pool Settings

| Database | Min Connections | Max Connections | Idle Timeout |
|----------|-----------------|-----------------|--------------|
| PostgreSQL | 2 | 10 | 300s |
| MySQL | 2 | 10 | 300s |
| SQLite | 1 | 1 | N/A |

### Environment Variables

```bash
# PostgreSQL
DATABASE_URL=postgres://user:pass@localhost:5432/dbname

# MySQL
DATABASE_URL=mysql://user:pass@localhost:3306/dbname

# SQLite
DATABASE_URL=sqlite:///path/to/db.sqlite

# Migrations directory
MIGRATIONS_DIR=./migrations
```

## Transaction Handling

The library wraps each migration in a single transaction. If any statement fails, the entire transaction is rolled back.

### PostgreSQL Transaction

```sql
-- This migration runs as a single transaction
BEGIN;

CREATE TABLE IF NOT EXISTS m_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_products_name ON m_products(name);

-- If either statement fails, both are rolled back
COMMIT;
```

### Savepoints for Complex Migrations

```sql
-- Use savepoints for partial rollback within a migration
BEGIN;

SAVEPOINT sp1;
CREATE TABLE IF NOT EXISTS m_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid()
);

-- If index creation fails, rollback to savepoint
SAVEPOINT sp2;
CREATE INDEX IF NOT EXISTS idx_items_name ON m_items(name);
-- ROLLBACK TO sp2; -- if needed

COMMIT;
```

## Schema Migrations Table

The library automatically creates and manages a `schema_migrations` table:

### PostgreSQL

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### MySQL

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version VARCHAR(14) PRIMARY KEY,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB;
```

### SQLite

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Querying Migration History

```sql
-- Get all applied versions
SELECT version FROM schema_migrations ORDER BY version ASC;

-- Get migration count
SELECT COUNT(*) FROM schema_migrations;

-- Get latest applied migration
SELECT version, applied_at FROM schema_migrations ORDER BY version DESC LIMIT 1;
```

## Multi-Database Support

For projects using multiple databases:

```go
// Go example
userDB := migration.NewMigrator("./migrations/users")
orderDB := migration.NewMigrator("./migrations/orders")
productDB := migration.NewMigrator("./migrations/products")

userDB.Run()
orderDB.Run()
productDB.Run()
```

```javascript
// Node.js example
const userM = new Migrator('./migrations/users');
const orderM = new Migrator('./migrations/orders');
const productM = new Migrator('./migrations/products');

userM.run();
orderM.run();
productM.run();
```

```python
# Python example
user_m = Migrator("./migrations/users")
order_m = Migrator("./migrations/orders")
product_m = Migrator("./migrations/products")

user_m.run()
order_m.run()
product_m.run()
```
