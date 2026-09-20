# MigrDB

[![Version](https://img.shields.io/badge/version-0.1.1-blue.svg)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

A high-performance SQL database migration and schema modeling library built with a Rust core engine, providing native bindings for Go, Node.js (npm), and Python, alongside ecosystem-aware model generation for Go, TypeScript, Python (SQLAlchemy 2.0), and C# (EF Core).

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Language Bindings                             │
├───────────────────────────────┬───────────────────┬─────────────────────┤
│  Go (Zero-CGO DLL / CGO)      │   Node.js (napi)  │    Python (PyO3)    │
└───────────────┬───────────────┴─────────┬─────────┴──────────┬──────────┘
                │                         │                    │
                └─────────────────────────┼────────────────────┘
                                          │
                               ┌──────────▼──────────┐
                               │   Rust FFI Layer    │
                               └──────────┬──────────┘
                                          │
                               ┌──────────▼──────────┐
                               │  Migration Engine   │
                               │  - Parser           │
                               │  - Tracker          │
                               │  - Migrator         │
                               │  - Codegen (DrawDB) │
                               └──────────┬──────────┘
                                          │
           ┌──────────────────────────────┼──────────────────────────────┐
           │                              │                              │
┌──────────▼──────────┐        ┌──────────▼──────────┐        ┌──────────▼──────────┐
│   Go Model Structs  │        │ TypeScript / Node   │        │ Python SQLAlchemy 2 │
│ - json & db tags    │        │ - Interfaces        │        │ - Mapped[...] types │
│ - Nullable pointers │        │ - Classes & types   │        │ - relationship()    │
│ - Navigation fields │        │ - Barrel index.ts   │        │ - DeclarativeBase   │
└─────────────────────┘        └─────────────────────┘        └─────────────────────┘
                                          │
                               ┌──────────▼──────────┐
                               │   C# EF Core 8/9    │
                               │ - Entity classes    │
                               │ - [Table], [Key]    │
                               │ - [ForeignKey]      │
                               │ - DbContext class   │
                               └─────────────────────┘
```

## Features

- **Universal Schema Sync & Auto-Migration** - Define schema visually in DrawDB (`drawdb.json`); MigraDB dynamically synchronizes live databases across **PostgreSQL (default)**, **MySQL**, and **SQLite** with transactional execution and automatic snapshot tracking.
- **Zero-Regeneration Database Switching** - Switching databases midway (e.g. SQLite in development to PostgreSQL/MySQL in production) requires **zero migration file regeneration**—MigraDB translates schema diffs into dialect-accurate DDL on the fly.
- **Automatic Directory Conventions & Customization** - Defaults to `./schema` and `./migrations` at the project root; automatically creates them if missing, or seamlessly adopts existing folders and custom paths.
- **Timestamp-based versioning** - Strict `YYYYMMDDHHmmss_slug.sql` chronological ordering
- **Transactional safety** - Each migration executes within an atomic transaction
- **Immutable history** - Never edit old migrations; create new migrations to roll forward or reverse changes
- **Idempotent SQL** - Native support for `IF NOT EXISTS` / `IF EXISTS` schema guards
- **Standalone CLI Tool** - Native `migradb` binary for running migrations, inspecting status, diffing schemas, and generating code without language dependencies
- **Ecosystem-aware Model Class Generation** - Entity Developer / EF Core-style code generation directly from DrawDB JSON schemas:
  - **Go**: Structs with `json` and `db` struct tags, pointer types for nullable columns, and relation navigation fields
  - **Node.js / TypeScript**: Clean TypeScript interfaces and classes with `Date`, `number`, and `string` mappings plus barrel exports (`index.ts`)
  - **Python**: Modern SQLAlchemy 2.0 declarative models using `Mapped[...]`, `mapped_column()`, and `relationship()`
  - **C#**: Full Entity Framework Core entity classes with data annotations (`[Table]`, `[Key]`, `[ForeignKey]`, `[InverseProperty]`) and a ready-to-use `MigraDbContext`
  - **Rust**: Serde-serializable structs with `Option<T>` for nullables and `sqlx::FromRow` derivation
  - **SQL DDL**: Complete `CREATE TABLE` scripts with constraints and foreign keys for PostgreSQL, MySQL, and SQLite
- **Direct Database Execution Helpers** - 1-line transaction-safe runners for live databases:
  - **Go**: `RunDB(ctx, db, "./migrations")` and `SyncDB(ctx, db, opts)` with standard `*sql.DB`
  - **Node.js**: `runOnDatabase(client, "./migrations")` and `syncDatabase(client, opts)` with `pg`, `mysql2`, or `better-sqlite3`
  - **Python**: `run_on_connection(conn, "./migrations")` and `sync_database(conn, ...)` with standard DB-API 2.0 or SQLite
- **DrawDB Schema Integration** - Direct ingestion of DrawDB JSON diagrams with automatic table prefix stripping (`m_`, `t_`, `sys_`, `map_`), singularization, and 1-to-many relationship mapping
- **Zero-CGO on Windows** - Go bindings load `migration_engine.dll` dynamically via `syscall.NewLazyDLL` without requiring GCC, MinGW, or CGO tooling
- **Multi-language support** - Bindings for Go, Node.js (via napi-rs), and Python (via PyO3) powered by a single compiled Rust core
- **Schema tracking** - Automatic `schema_migrations` audit table management

---


## Installation & Quick Start

### Go

```bash
go get github.com/MuchammadTedyA/migradb/go
```

The Go package supports **Zero-CGO dynamic loading on Windows** (loads `migration_engine.dll` without MinGW/GCC) and **standard CGO on Linux/macOS**.

#### Run Migrations Directly on Your Database (`RunDB`)
Works with any standard Go SQL driver (PostgreSQL via `lib/pq` or `pgx/stdlib`, MySQL via `go-sql-driver/mysql`, SQLite via `go-sqlite3`):

```go
package main

import (
    "context"
    "database/sql"
    "fmt"
    "log"

    _ "github.com/lib/pq"
    migration "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    ctx := context.Background()

    // 1. Connect to database
    db, err := sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    // 2. Check migration status
    status, err := migration.StatusDB(ctx, db, "./migrations")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Total migrations tracked: %d\n", len(status.Migrations))

    // 3. Apply all pending migrations in atomic transactions
    result, err := migration.RunDB(ctx, db, "./migrations")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Successfully applied %d migration(s)!\n", result.Applied)

    // 4. Generate Go model structs from DrawDB visual schema export
    m := migration.NewMigrator("./migrations")
    defer m.Close()
    m.GenerateModels("schema/drawdb.json", "go", "./internal/models", "models")
}
```

### Node.js (npm)

```bash
npm install migradb
```

#### Run Migrations Directly on Your Database
`migradb` includes built-in live database runners for `pg` (PostgreSQL), `mysql2` (MySQL), and `better-sqlite3` (SQLite):

```javascript
const { Client } = require('pg');
const { runOnDatabase, statusOnDatabase, generateModels } = require('migradb');

async function main() {
    const client = new Client({ connectionString: process.env.DATABASE_URL });
    await client.connect();

    // Run all pending migrations inside a transaction
    const result = await runOnDatabase(client, './migrations');
    console.log(`Applied ${result.applied} migrations!`);

    // Check status anytime
    const status = await statusOnDatabase(client, './migrations');
    console.log(`Total migrations tracked: ${status.migrations.length}`);

    await client.end();

    // Generate TypeScript interfaces and models from DrawDB JSON diagram
    generateModels('schema/drawdb.json', 'node', './src/models');
}

main().catch(console.error);
```

> **CLI Shortcut**: You can also use the bundled CLI directly with `npx migradb`:
> ```bash
> npx migradb init --dir ./migrations
> npx migradb create add_users --dir ./migrations
> npx migradb generate --schema drawdb.json --lang node --out ./src/models
> ```

### Python

```bash
pip install migradb
```

#### Run Migrations Directly on Your Database
`migradb` supports standard Python DB-API 2.0 connections (`psycopg2`, `sqlite3`, `pymysql`):

```python
import psycopg2
from migradb import run_on_connection, status_on_connection, generate_models

# Connect to database
conn = psycopg2.connect("dbname=myapp user=postgres password=secret host=localhost")

# Run all pending migrations inside an atomic transaction
result = run_on_connection(conn, "./migrations")
if result.success:
    print(f"Applied {result.applied} migrations")
else:
    print(f"Migration error: {result.error}")

# Check status
status = status_on_connection(conn, "./migrations")
for s in status.migrations:
    print(f"[{'APPLIED' if s.applied else 'PENDING'}] {s.version} - {s.name}")

conn.close()

# Generate SQLAlchemy 2.0 models from DrawDB diagram
generate_models("schema/drawdb.json", "python", "./app/models.py")
```

---

## Universal Schema Sync & Auto-Migration (Zero Regeneration)

MigraDB features a universal runtime synchronization engine (**Approach 1**). You design your database schema visually once in [DrawDB](https://drawdb.app) and save the JSON export in `./schema/drawdb.json`. At application startup or deployment, MigraDB dynamically translates schema diffs into the target database dialect and applies them safely inside an atomic transaction.

### How It Works

```
┌────────────────────────┐
│   schema/drawdb.json   │  (Single Source of Truth)
└───────────┬────────────┘
            │
            ▼
┌────────────────────────┐
│  MigraDB Engine Diff   │◄──── Compares against:
└───────────┬────────────┘      • schema/drawdb_snapshot.json
            │                   • migrations/.schema_snapshot.json
            │
            ├───────────────┬───────────────┐
            ▼               ▼               ▼
     [PostgreSQL]        [MySQL]        [SQLite]
      (Default)
            │               │               │
            └───────────────┼───────────────┘
                            │
                            ▼
          ┌──────────────────────────────────┐
          │  Atomic Database Transaction     │
          │  - Executes dialect-specific DDL │
          │  - Records schema_migrations     │
          │  - Saves audit .sql migration    │
          │  - Updates snapshots             │
          └──────────────────────────────────┘
```

### Key Capabilities

1. **PostgreSQL Default with Full MySQL & SQLite Support**:
   - Dialect is **PostgreSQL** by default if unspecified.
   - MySQL (using backtick escaping, `MODIFY COLUMN`, and `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4`) and SQLite (using portable table/column alterations) are fully supported first-class dialects.
2. **Switching Databases Midway Requires ZERO Migration Regeneration**:
   - If you start development with SQLite and later switch midway to MySQL or PostgreSQL in production, **you do not need to rewrite or regenerate migrations**.
   - Simply point your application to the new database; MigraDB evaluates the schema against the target database and generates the exact DDL statements required for that engine on the fly.
3. **Folder Auto-Creation & Customization**:
   - By default, MigraDB checks for `./schema` and `./migrations` in your project root. If either folder does not exist, MigraDB automatically creates it. If it already exists, files are placed cleanly inside.
   - Custom directories can be specified at any time (`migrations_dir`, `schema_dir`, `--dir`, `--schema-dir`).
4. **Audit Trail & Snapshot Integrity**:
   - Every synchronization saves an immutable timestamped audit migration (`migrations/YYYYMMDDHHmmss_sync_drawdb.sql`) and maintains snapshots (`schema/drawdb_snapshot.json` and `migrations/.schema_snapshot.json`) to track state across team members and CI/CD pipelines.

---

### Usage: Go (`SyncDB`)

```go
package main

import (
    "context"
    "database/sql"
    "fmt"
    "log"

    _ "github.com/lib/pq" // or github.com/go-sql-driver/mysql or modernc.org/sqlite
    migration "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    ctx := context.Background()

    // 1. Connect to your database (PostgreSQL, MySQL, or SQLite)
    db, err := sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    // 2. Synchronize schema directly from drawdb.json
    res, err := migration.SyncDB(ctx, db, migration.SyncOptions{
        SchemaPath:        "./schema/drawdb.json", // Default schema file
        MigrationsDir:     "./migrations",        // Auto-created if missing
        Dialect:           migration.DialectPostgres, // "postgres" (default), "mysql", or "sqlite"
        SaveMigrationFile: true,                  // Generates audit .sql migration
        MigrationName:     "sync_schema",         // Name slug for audit migration
    })
    if err != nil {
        log.Fatalf("Sync failed: %v", err)
    }

    if res.IsEmpty {
        fmt.Println("Database schema is already up to date!")
    } else {
        fmt.Printf("Successfully applied %d DDL statement(s)!\n", res.Applied)
        fmt.Printf("Audit migration saved: %s\n", res.MigrationPath)
    }
}
```

---

### Usage: Node.js (`syncDatabase`)

```javascript
const { Client } = require('pg'); // or require('mysql2/promise') or require('better-sqlite3')
const { syncDatabase, detectDialect } = require('migradb');

async function main() {
    const client = new Client({ connectionString: process.env.DATABASE_URL });
    await client.connect();

    // Synchronize schema against drawdb.json
    const res = await syncDatabase(client, {
        schema: './schema/drawdb.json',         // Default schema file
        migrationsDir: './migrations',         // Auto-created if missing
        schemaDir: './schema',                 // Auto-created if missing
        // dialect: 'postgres',                // Auto-detected or explicitly: 'postgres', 'mysql', 'sqlite'
        saveMigrationFile: true,               // Generates audit .sql migration
        migrationName: 'sync_schema',          // Name slug for audit migration
    });

    if (res.isEmpty) {
        console.log('Database schema is already up to date!');
    } else if (res.success) {
        console.log(`Applied ${res.applied} DDL statement(s)!`);
        console.log(`Audit migration: ${res.migrationPath}`);
    } else {
        console.error(`Sync error: ${res.error}`);
    }

    await client.end();
}

main().catch(console.error);
```

---

### Usage: Python (`sync_database`)

```python
import psycopg2 # or pymysql or sqlite3
from migradb import sync_database, detect_dialect

conn = psycopg2.connect("dbname=myapp user=postgres password=secret host=localhost")

# Synchronize schema against drawdb.json
res = sync_database(
    conn=conn,
    schema="./schema/drawdb.json",         # Default schema file
    migrations_dir="./migrations",         # Auto-created if missing
    schema_dir="./schema",                 # Auto-created if missing
    # dialect="postgres",                  # Auto-detected or explicitly: 'postgres', 'mysql', 'sqlite'
    save_migration_file=True,              # Generates audit .sql migration
    migration_name="sync_schema",          # Name slug for audit migration
)

if res.is_empty:
    print("Database schema is already up to date!")
elif res.success:
    print(f"Applied {res.applied} DDL statement(s)!")
    print(f"Audit migration: {res.migration_path}")
else:
    print(f"Sync error: {res.error}")

conn.close()
```

---

### CLI Incremental Diffing (`migradb diff`)

You can generate incremental SQL migrations from your DrawDB visual schema at any time using the CLI:

```bash
# PostgreSQL (default)
migradb diff --schema ./schema/drawdb.json --name add_user_profiles

# MySQL
migradb diff --schema ./schema/drawdb.json --name add_user_profiles --dialect mysql

# SQLite
migradb diff --schema ./schema/drawdb.json --name add_user_profiles --dialect sqlite

# Specify custom directories
migradb diff --schema ./custom/drawdb.json --dir ./custom_migrations --schema-dir ./custom_schema

# Force full baseline regeneration
migradb diff --schema ./schema/drawdb.json --full
```

With Node.js / `npx`:
```bash
npx migradb diff --schema ./schema/drawdb.json --dialect postgres
npx migradb diff --schema ./schema/drawdb.json --dialect mysql
```

---


## Migration File Format

Migration files follow the naming convention `<timestamp>_<slug>.sql`:

- **Timestamp**: 14-digit format `YYYYMMDDHHmmss`
- **Slug**: Lowercase description with words separated by underscores

Example structure:
```
migrations/
├── 20260901000000_initial_schema.sql
├── 20260901000100_add_branches_table.sql
└── 20260901000200_add_employees_table.sql
```

Sample migration content (`20260901000100_add_branches_table.sql`):
```sql
CREATE TABLE IF NOT EXISTS m_branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES m_companies(id) ON DELETE RESTRICT,
    name VARCHAR(200) NOT NULL,
    code VARCHAR(50) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_branches_company_id ON m_branches(company_id);
```

---

## Model Class Generation (DrawDB to Code)

MigrDB includes an Entity Developer / EF Core-style code generator that parses DrawDB diagram export files and produces clean, idiomatically typed model files.

### Table Name Conventions

The generator automatically strips known architectural prefixes and singularizes table names:

| Table Name in Database | Detected Prefix | Generated Entity Name | Go / TS File |
|------------------------|-----------------|-----------------------|--------------|
| `m_companies`          | `m_` (Master)   | `Company`             | `company.go` / `company.ts` / `Company.cs` |
| `m_branches`           | `m_` (Master)   | `Branch`              | `branch.go` / `branch.ts` / `Branch.cs` |
| `t_orders`             | `t_` (Trans.)   | `Order`               | `order.go` / `order.ts` / `Order.cs` |
| `map_user_roles`       | `map_` (Junct.) | `UserRole`            | `user_role.go` / `user_role.ts` |
| `sys_configurations`   | `sys_` (System) | `Configuration`       | `configuration.go` / `Configuration.cs` |

### Target Language Support

| Target Flag | Output Description | Key Features |
|-------------|--------------------|--------------|
| `"go"` / `"golang"` | Go struct files (`.go`) | `json` & `db` tags, pointer types (`*string`, `*time.Time`) for nullable fields, relation navigation pointers |
| `"node"` / `"ts"` | TypeScript files (`.ts`) + `index.ts` | TypeScript interfaces, optional `?` properties, `Date` types, and centralized barrel export |
| `"python"` / `"py"` | Python `models.py` | SQLAlchemy 2.0 declarative models (`Mapped[T]`, `mapped_column`, `relationship`) |\
| `"rust"` / `"rs"` | Rust struct files (`.rs`) + `mod.rs` | `serde` serialization, `Option<T>` for nullable columns, and `sqlx::FromRow` derivation |
| `"sql"` / `"ddl"` | SQL DDL scripts (`schema.sql`) | Idempotent `CREATE TABLE IF NOT EXISTS`, constraints, and foreign keys for PostgreSQL, MySQL, and SQLite |

---

## CLI Tools

### Standalone Binary (`migradb`)
A compiled standalone binary is available for running migrations, inspecting schema status, and generating models without any language dependencies:

```bash
# Initialize a new migrations project
migradb init --dir ./migrations

# Create a new timestamped migration file
migradb create add_users_table --dir ./migrations

# Inspect migration status table
migradb status --dir ./migrations

# Generate Rust models from DrawDB diagram
migradb generate --schema drawdb.json --lang rust --out ./src/models

# Generate SQL DDL for PostgreSQL from DrawDB diagram
migradb generate --schema drawdb.json --lang sql --pkg postgres --out ./migrations
```

### Node.js CLI via `npx migradb`
If you are in a JavaScript / TypeScript project, `npx migradb` is included out of the box when you install the `migradb` npm package:

```bash
npx migradb init --dir ./migrations
npx migradb create add_users_table --dir ./migrations
npx migradb status --dir ./migrations
npx migradb generate --schema drawdb.json --lang node --out ./src/models
```

---

## API Reference

### Direct Database Execution Helpers
Transaction-safe 1-line execution helpers for live databases:

- **Node.js**:
  - `syncDatabase(client, options)`: Dynamically syncs database against DrawDB visual schema with multi-dialect DDL translation. Returns `Promise<SyncResult>`.
  - `detectDialect(client)`: Helper that infers `'postgres'`, `'mysql'`, or `'sqlite'` from database client object.
  - `runOnDatabase(client, migrationsDir)`: Runs pending migrations in transaction. Supports `pg`, `mysql2`, `better-sqlite3`. Returns `Promise<RunResult>`.
  - `statusOnDatabase(client, migrationsDir)`: Queries migration history. Returns `Promise<StatusResult>`.
- **Python**:
  - `sync_database(conn, ...)`: Dynamically syncs database against DrawDB visual schema with multi-dialect DDL translation. Returns `ResultDict`.
  - `detect_dialect(conn)`: Helper that infers `'postgres'`, `'mysql'`, or `'sqlite'` from DB-API 2.0 connection.
  - `run_on_connection(conn, migrations_dir)`: Runs pending migrations in transaction. Supports standard DB-API 2.0 (`psycopg2`, `sqlite3`, `pymysql`). Returns `RunResult`.
  - `status_on_connection(conn, migrations_dir)`: Queries migration history. Returns `StatusResult`.
- **Go**:
  - `SyncDB(ctx, db, opts)`: Dynamically syncs database against DrawDB visual schema with multi-dialect DDL translation. Returns `(*SyncResult, error)`.
  - `RunDB(ctx, db, migrationsDir)`: Runs pending migrations using standard `*sql.DB`. Returns `(*RunResult, error)`.
  - `StatusDB(ctx, db, migrationsDir)`: Queries migration history. Returns `(*StatusResult, error)`.

### Migrator Methods

#### `new Migrator(migrationsDir: string)` / `NewMigrator(migrationsDir string)`
Creates a new Migrator instance pointing to the migration files directory.

#### `run()` / `Run()`
Calculates pending migrations in chronological order against the internal tracker.
- **Returns**: Result object containing `success`, count of `applied` migrations, list of `migrations`, and optional `error`.

#### `status()` / `Status()`
Inspects all migration files against the migration history.
- **Returns**: Result object containing `success` and a list of migrations with `version`, `name`, `applied` (boolean), and `applied_at` timestamp.

#### `create(name, content)` / `Create(name, content string)`
Creates a new migration file with the current UTC timestamp: `<timestamp>_<name>.sql`.
- **Returns**: Result object containing `success`, generated file `path`, and optional `error`.

#### `removePending()` / `RemovePending()`
Safely removes the most recent migration file **only** if it has not yet been applied to the database.
- **Returns**: Result object containing `success`, `removed` path, informational message, and optional `error`.

#### `generateModels(schemaPath, targetLang, outputDir, pkgName)` / `generate_models(...)`
Generates strongly-typed entity model classes from a DrawDB JSON export file. Available as a standalone function and as a `Migrator` method across Go, Node.js, and Python.
- **Parameters**:
  - `schemaPath`: Path to the DrawDB JSON export file
  - `targetLang`: Language target (`"go"`, `"node"`, `"python"`, `"rust"`, `"csharp"`, `"sql"`)
  - `outputDir`: Target directory where generated files will be written
  - `pkgName`: Package name (for Go) or Namespace (for C#)
- **Returns**: Result object containing `success`, file `count` (or list of `files`), and optional `error`.

---

## Building from Source

### Prerequisites

- **Rust** 1.70+ (`cargo`)
- **Go** 1.21+ (for Go bindings)
- **Node.js** 18+ and `npm` (for npm package)
- **Python** 3.8+ (for Python package)

### Automated Build Scripts

```bash
# Windows PowerShell
.\build.ps1

# Unix / macOS
./build.sh
```

### Manual Build Steps

```bash
# 1. Build Rust core engine
cargo build --release

# 2. Build Go bindings
cd go
go build ./...
go test -v ./...
cd ..

# 3. Build Node.js bindings
cd npm
npm install
npm run build
cd ..

# 4. Build Python bindings
cd python
pip install maturin
maturin develop
cd ..
```

---

## Documentation Directory

Comprehensive documentation is available in the [`docs/`](docs/) directory:

- **Guides**:
  - [Migration Guide](docs/guides/migrations.md) - Writing safe migrations and schema conventions
  - [Go Guide](docs/guides/go.md) - Go integration, Windows Zero-CGO loading, and code generation
  - [Node.js Guide](docs/guides/nodejs.md) - Node.js and TypeScript usage
  - [Python Guide](docs/guides/python.md) - Python and async usage
  - [Database Guide](docs/guides/database.md) - PostgreSQL, MySQL, and SQLite configuration
- **API References**:
  - [Go API Reference](docs/api/go.md)
  - [Node.js API Reference](docs/api/nodejs.md)
  - [Python API Reference](docs/api/python.md)
- **Examples**:
  - [Go Examples](docs/examples/go.md)
  - [Node.js Examples](docs/examples/nodejs.md)
  - [Python Examples](docs/examples/python.md)

---

## Design Principles

1. **Immutable History** - Never modify previously applied migrations; always create a new migration to change schema.
2. **Idempotent SQL** - Protect schema statements with `IF NOT EXISTS` / `IF EXISTS` guards.
3. **Transactional Safety** - Each migration runs in an isolated database transaction.
4. **Chronological Versioning** - Timestamp prefixes guarantee unambiguous order across distributed teams.
5. **Roll-Forward Architecture** - Resolve issues by pushing forward migrations rather than running destructive down migrations in production.

---

## License

[MIT](LICENSE) © Muchammad Tedy
