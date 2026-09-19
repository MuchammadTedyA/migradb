# MigrDB

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](CHANGELOG.md)
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

- **Timestamp-based versioning** - Strict `YYYYMMDDHHmmss_slug.sql` chronological ordering
- **Transactional safety** - Each migration executes within an atomic transaction
- **Immutable history** - Never edit old migrations; create new migrations to roll forward or reverse changes
- **Idempotent SQL** - Native support for `IF NOT EXISTS` / `IF EXISTS` schema guards
- **Standalone CLI Tool** - Native `migradb` binary for running migrations, inspecting status, and generating code without language dependencies
- **Ecosystem-aware Model Class Generation** - Entity Developer / EF Core-style code generation directly from DrawDB JSON schemas:
  - **Go**: Structs with `json` and `db` struct tags, pointer types for nullable columns, and relation navigation fields
  - **Node.js / TypeScript**: Clean TypeScript interfaces and classes with `Date`, `number`, and `string` mappings plus barrel exports (`index.ts`)
  - **Python**: Modern SQLAlchemy 2.0 declarative models using `Mapped[...]`, `mapped_column()`, and `relationship()`
  - **C#**: Full Entity Framework Core entity classes with data annotations (`[Table]`, `[Key]`, `[ForeignKey]`, `[InverseProperty]`) and a ready-to-use `MigraDbContext`
  - **Rust**: Serde-serializable structs with `Option<T>` for nullables and `sqlx::FromRow` derivation
  - **SQL DDL**: Complete `CREATE TABLE` scripts with constraints and foreign keys for PostgreSQL, MySQL, and SQLite
- **Direct Database Execution Helpers** - 1-line transaction-safe runners for live databases:
  - **Go**: `RunDB(ctx, db, "./migrations")` with standard `*sql.DB`
  - **Node.js**: `runOnDatabase(client, "./migrations")` with `pg`, `mysql2`, or `better-sqlite3`
  - **Python**: `run_on_connection(conn, "./migrations")` with standard DB-API 2.0 or SQLite
- **DrawDB Schema Integration** - Direct ingestion of DrawDB JSON diagrams with automatic table prefix stripping (`m_`, `t_`, `sys_`, `map_`), singularization, and 1-to-many relationship mapping
- **Zero-CGO on Windows** - Go bindings load `migration_engine.dll` dynamically via `syscall.NewLazyDLL` without requiring GCC, MinGW, or CGO tooling
- **Multi-language support** - Bindings for Go, Node.js (via napi-rs), and Python (via PyO3) powered by a single compiled Rust core
- **Schema tracking** - Automatic `schema_migrations` audit table management

---

## Installation & Quick Start

### Go

The Go package supports **Zero-CGO dynamic loading on Windows** and **standard CGO on Linux/macOS**.

```bash
# 1. Build the Rust core engine
cargo build --release

# 2. Linux / macOS only: export library path (Windows locates DLL automatically)
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH
```

```go
package main

import (
    "fmt"
    "log"

    migration "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    // Initialize migrator with migrations directory
    m := migration.NewMigrator("./migrations")
    if m == nil {
        log.Fatal("Failed to initialize migrator")
    }
    defer m.Close()

    // Run all pending migrations
    result, err := m.Run()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Applied %d migrations\n", result.Applied)

    // Generate Go model classes from a DrawDB visual schema export
    modelResult, err := m.GenerateModels(
        "schema/drawdb.json", // DrawDB JSON file
        "go",                 // Target language: go, node, python, or csharp
        "./internal/models",  // Target directory
        "models",             // Package or namespace name
    )
    if err != nil {
        log.Fatalf("Model generation failed: %v", err)
    }
    fmt.Printf("Generated %d model files in ./internal/models\n", modelResult.Count)
}
```

### Node.js (npm)

```bash
npm install migradb
```

```javascript
const { Migrator } = require('migradb');

const m = new Migrator('./migrations');

// Check status
const status = m.status();
if (status.success) {
    status.migrations.forEach(s => {
        console.log(`[${s.applied ? 'APPLIED' : 'PENDING'}] ${s.version} - ${s.name}`);
    });
}

// Run pending migrations
const result = m.run();
if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
} else {
    console.error(`Migration failed: ${result.error}`);
}
```

### Python

```bash
pip install migradb
```

```python
from migradb import Migrator

m = Migrator("./migrations")

# Check status
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

## Standalone CLI Tool

MigraDB includes a standalone compiled CLI tool (`migradb`):

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

## API Reference

### Migrator Methods

#### `new Migrator(migrationsDir: string)` / `NewMigrator(migrationsDir string)`
Creates a new Migrator instance pointing to the migration files directory.

#### `run()` / `Run()`
Executes all pending migrations in chronological order within individual transactions.
- **Returns**: Result object containing `success`, count of `applied` migrations, list of `migrations`, and optional `error`.

#### `status()` / `Status()`
Inspects all migration files against the database migration history.
- **Returns**: Result object containing `success` and a list of migrations with `version`, `name`, `applied` (boolean), and `applied_at` timestamp.

#### `create(name, content)` / `Create(name, content string)`
Creates a new migration file with the current UTC timestamp: `<timestamp>_<name>.sql`.
- **Returns**: Result object containing `success`, generated file `path`, and optional `error`.

#### `removePending()` / `RemovePending()`
Safely removes the most recent migration file **only** if it has not yet been applied to the database.
- **Returns**: Result object containing `success`, `removed` path, informational message, and optional `error`.

#### `generateModels(schemaPath, targetLang, outputDir, pkgName)` / `GenerateModels(...)`
Generates strongly-typed entity model classes from a DrawDB JSON export file.
- **Parameters**:
  - `schemaPath`: Path to the DrawDB JSON export file
  - `targetLang`: Language target (`"go"`, `"node"`, `"python"`, or `"csharp"`)
  - `outputDir`: Target directory where generated files will be written
  - `pkgName`: Package name (for Go) or Namespace (for C#)
- **Returns**: Result object containing `success`, file `count`, list of written `files`, and optional `error`.

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
