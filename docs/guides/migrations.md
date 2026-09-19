# Migration Guide

This guide explains how to create, manage, and execute database migrations with the MigrDB.

## Table of Contents

- [Overview](#overview)
- [File Naming Convention](#file-naming-convention)
- [Creating Migrations](#creating-migrations)
- [Writing Migration SQL](#writing-migration-sql)
- [Running Migrations](#running-migrations)
- [Checking Status](#checking-status)
- [Rolling Back](#rolling-back)
- [Best Practices](#best-practices)

## Overview

Migrations are SQL files that modify your database schema. Each migration is executed in order based on its timestamp version, and tracked in a `schema_migrations` table.

## File Naming Convention

Migration files follow the format:

```
YYYYMMDDHHmmss_slug.sql
```

| Component | Description | Example |
|-----------|-------------|---------|
| `YYYYMMDDHHmmss` | 14-digit timestamp | `20260901000000` |
| `slug` | Lowercase description with underscores | `create_users_table` |

### Examples

```
migrations/
├── 20260901000000_initial_schema.sql
├── 20260901000100_add_branches_table.sql
├── 20260901000200_add_employees_table.sql
└── 20260901000300_add_email_index.sql
```

## Creating Migrations

### Using the Library API

#### Go

```go
result, err := m.Create("add products table", `
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL,
        price DECIMAL(10,2) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
`)
```

#### Node.js

```javascript
const result = m.create('add products table', `
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL,
        price DECIMAL(10,2) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
`);
```

#### Python

```python
result = m.create("add products table", """
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL,
        price DECIMAL(10,2) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
""")
```

### Manual Creation

Create a file with the timestamp prefix:

```bash
# Format: YYYYMMDDHHmmss_description.sql
touch migrations/20260901000000_initial_schema.sql
```

## Writing Migration SQL

### Recommended Patterns

#### Create Table

```sql
CREATE TABLE IF NOT EXISTS m_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### Add Column

```sql
ALTER TABLE m_users
ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
```

#### Create Index

```sql
CREATE INDEX IF NOT EXISTS idx_m_users_email ON m_users(email);
CREATE UNIQUE INDEX IF NOT EXISTS idx_m_users_username ON m_users(username);
```

#### Add Foreign Key

```sql
ALTER TABLE t_orders DROP CONSTRAINT IF EXISTS fk_orders_user;
ALTER TABLE t_orders
ADD CONSTRAINT fk_orders_user
FOREIGN KEY (user_id) REFERENCES m_users(id)
ON DELETE CASCADE;
```

### Idempotent SQL

Always use `IF NOT EXISTS` and `IF EXISTS` guards:

```sql
-- Good: Idempotent
CREATE TABLE IF NOT EXISTS m_products (...);
CREATE INDEX IF NOT EXISTS idx_products_name ON m_products(name);

-- Bad: Will fail on second run
CREATE TABLE m_products (...);
CREATE INDEX idx_products_name ON m_products(name);
```

## Running Migrations

### Automatic Execution

```go
// Go
result, err := m.Run()
```

```javascript
// Node.js
const result = m.run();
```

```python
# Python
result = m.run()
```

### Result Structure

```json
{
    "success": true,
    "applied": 3,
    "migrations": [
        {"version": "20260901000000", "name": "initial_schema"},
        {"version": "20260901000100", "name": "add_branches_table"},
        {"version": "20260901000200", "name": "add_employees_table"}
    ]
}
```

## Checking Status

```go
// Go
status, err := m.Status()
for _, s := range status.Migrations {
    fmt.Printf("[%s] %s - %s\n", s.Applied, s.Version, s.Name)
}
```

```javascript
// Node.js
const status = m.status();
status.migrations.forEach(s => {
    console.log(`[${s.applied ? 'APPLIED' : 'PENDING'}] ${s.version} - ${s.name}`);
});
```

```python
# Python
status = m.status()
for s in status.migrations:
    status_str = "APPLIED" if s.applied else "PENDING"
    print(f"[{status_str}] {s.version} - {s.name}")
```

### Status Output Example

```
[APPLIED] 20260901000000 - initial_schema
[APPLIED] 20260901000100 - add_branches_table
[PENDING] 20260901000200 - add_employees_table
```

## Rolling Back

This library does **not** support traditional down migrations. Instead, create a new migration to reverse changes:

### Example: Rolling Back a Column Addition

**Original migration** (`20260901000100_add_phone_to_users.sql`):
```sql
ALTER TABLE m_users ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
```

**Rollback migration** (`20260901000200_remove_phone_from_users.sql`):
```sql
ALTER TABLE m_users DROP COLUMN IF EXISTS phone;
```

### Removing Pending Migrations

If a migration hasn't been applied yet, you can remove the file:

```go
// Go
result, err := m.RemovePending()
```

```javascript
// Node.js
const result = m.removePending();
```

```python
# Python
result = m.remove_pending()
```

## Best Practices

### 1. Keep Migrations Small

Each migration should make a single, focused change:

```sql
-- Good: Single change
CREATE TABLE IF NOT EXISTS m_products (...);

-- Avoid: Multiple unrelated changes
CREATE TABLE IF NOT EXISTS m_products (...);
CREATE TABLE IF NOT EXISTS m_categories (...);
ALTER TABLE m_users ADD COLUMN age INT;
```

### 2. Use Descriptive Names

```sql
-- Good
20260901000000_create_users_table.sql
20260901000100_add_email_index_to_users.sql

-- Bad
20260901000000_migration1.sql
20260901000100_fix.sql
```

### 3. Test Migrations

Always test migrations on a development database before applying to production.

### 4. Version Control

Commit migration files to version control. They are part of your application's history.

### 5. Never Modify Applied Migrations

Once a migration has been applied to any environment, never modify it. Create a new migration instead.

### 6. Use Transactions

The library automatically wraps each migration in a transaction. Ensure your SQL is compatible with transactional execution.

### 7. Follow Naming Conventions

Use standard SQL table prefix conventions:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `m_` | Master data | `m_users`, `m_products` |
| `t_` | Transactions | `t_orders`, `t_payments` |
| `map_` | Mapping/junction | `map_user_roles` |
| `sys_` | System/configuration | `sys_settings` |

### 8. Generating Model Classes from Schema (DrawDB to Code)

MigrDB provides an integrated visual-schema-to-code generator. You can design your database visually using [DrawDB](https://drawdb.app/), export the diagram as JSON, and generate production-ready, strongly-typed model classes across four major language ecosystems:

```go
m := migration.NewMigrator("migrations")
defer m.Close()

// Automatically strips prefixes (m_, t_, sys_, map_), singularizes names, and links foreign keys
res, err := m.GenerateModels(
    "schema/drawdb.json", // Path to exported DrawDB JSON
    "go",                 // Target: "go", "node", "python", "csharp"
    "./internal/models",  // Output directory
    "models",             // Package name / Namespace
)
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Generated %d model files in ./internal/models\n", res.Count)
```

#### Supported Output Targets:

1. **Go (`"go"`)**:
   - Generates individual `.go` files per entity.
   - Adds `json:"<field>"` and `db:"<field>"` struct tags.
   - Employs pointer types (`*string`, `*time.Time`, etc.) for nullable fields.
   - Generates typed navigation pointers for 1-to-many foreign keys (e.g. `Company *Company` with `db:"-"`).

2. **TypeScript / Node.js (`"node"`)**:
   - Generates `.ts` files with exported interfaces and classes.
   - Maps SQL datatypes to TypeScript primitives (`string`, `number`, `Date`, `boolean`).
   - Generates a root `index.ts` barrel file exporting all entities.

3. **Python (`"python"`)**:
   - Generates modern SQLAlchemy 2.0 declarative models in `models.py`.
   - Uses `Mapped[...]`, `mapped_column(...)`, and `relationship(...)` navigation bindings.
   - Inherits from a shared `DeclarativeBase`.

4. **C# / .NET (`"csharp"`)**:
   - Generates partial entity classes with EF Core data annotations (`[Table]`, `[Key]`, `[ForeignKey]`, `[InverseProperty]`).
   - Generates a ready-to-register `MigraDbContext` class with all `DbSet<T>` properties configured.

