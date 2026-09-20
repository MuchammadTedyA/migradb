# Go Guide

Complete guide for using MigrDB in Go projects.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Integration with Web Frameworks](#integration-with-web-frameworks)
- [Database Connection](#database-connection)

## Installation

### Prerequisites

- Go 1.21 or later
- Rust 1.70+ (for building the native library)
- **Windows**: No C compiler / CGO required! Loads `migration_engine.dll` dynamically.
- **Linux/macOS**: CGO enabled (GCC/Clang) for native CGO bindings.

### Step 1: Build the Native Library

```bash
cd migradb
cargo build --release
```

Output:
- Windows: `target/release/migration_engine.dll`
- Linux: `target/release/libmigration_engine.so`
- macOS: `target/release/libmigration_engine.dylib`

### Step 2: Install / Link the Go Package

In your project's `go.mod`:

```bash
# Link local development copy
go mod edit -replace github.com/MuchammadTedyA/migradb/go=../migradb/go
go mod tidy
```

### Step 3: Library Resolution

- **Windows**: `migradb/go` automatically searches:
  1. `MIGRATION_ENGINE_DLL` environment variable (if set)
  2. Current directory: `migration_engine.dll`
  3. `../target/release/migration_engine.dll`
  4. `../../migradb/target/release/migration_engine.dll`
- **Linux/macOS**: Set `LD_LIBRARY_PATH`:
  ```bash
  export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH
  ```

## Quick Start

MigraDB provides both **direct database runners** (`RunDB`, `StatusDB`) for executing migrations against standard Go `*sql.DB` connections, and the **core `Migrator`** for file management and DrawDB model generation.

### 1. Running Migrations on a Real Database (Recommended)

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

    // 1. Open database connection
    db, err := sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    // 2. Inspect migration status (applied vs pending)
    status, err := migration.StatusDB(ctx, db, "./migrations")
    if err != nil {
        log.Fatal(err)
    }
    for _, m := range status.Migrations {
        state := "PENDING"
        if m.Applied {
            state = "APPLIED"
        }
        fmt.Printf("[%s] %s - %s\n", state, m.Version, m.Name)
    }

    // 3. Apply all pending migrations in atomic transactions
    result, err := migration.RunDB(ctx, db, "./migrations")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Successfully applied %d migration(s)!\n", result.Applied)
}
```

### 2. Generating Go Models from DrawDB (`drawdb.json`)

```go
package main

import (
    "fmt"
    "log"

    migration "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    // Generate Go structs with json/db tags and relationship pointers
    res, err := m.GenerateModels("schema/drawdb.json", "go", "./internal/models", "models")
    if err != nil {
        log.Fatalf("Model generation failed: %v", err)
    }
    fmt.Printf("Generated %d model files in ./internal/models\n", res.Count)
}
```

## API Reference

### Types

#### `Migrator`

The main struct for managing migrations.

```go
type Migrator struct {
    handle unsafe.Pointer
}
```

#### `Migration`

Represents a single migration.

```go
type Migration struct {
    Version string `json:"version"`
    Name    string `json:"name"`
}
```

#### `MigrationStatus`

Represents the status of a migration.

```go
type MigrationStatus struct {
    Version   string  `json:"version"`
    Name      string  `json:"name"`
    Applied   bool    `json:"applied"`
    AppliedAt *string `json:"applied_at,omitempty"`
}
```

#### `RunResult`

Result from running migrations.

```go
type RunResult struct {
    Success    bool        `json:"success"`
    Applied    int         `json:"applied"`
    Migrations []Migration `json:"migrations"`
    Error      string      `json:"error,omitempty"`
}
```

#### `StatusResult`

Result from checking migration status.

```go
type StatusResult struct {
    Success    bool              `json:"success"`
    Migrations []MigrationStatus `json:"migrations"`
    Error      string            `json:"error,omitempty"`
}
```

#### `CreateResult`

Result from creating a migration.

```go
type CreateResult struct {
    Success bool   `json:"success"`
    Path    string `json:"path"`
    Error   string `json:"error,omitempty"`
}
```

#### `RemoveResult`

Result from removing a pending migration.

```go
type RemoveResult struct {
    Success bool   `json:"success"`
    Removed string `json:"removed,omitempty"`
    Message string `json:"message,omitempty"`
    Error   string `json:"error,omitempty"`
}
```

### Functions

#### `RunDB(ctx context.Context, db *sql.DB, migrationsDir string) (*RunResult, error)`

Executes all pending migrations directly against any standard Go `*sql.DB` connection within atomic transactions. Automatically creates and updates the `schema_migrations` audit table.

**Parameters:**
- `ctx` - Context for query cancellation and timeouts
- `db` - Standard library `*sql.DB` connection pool
- `migrationsDir` - Directory containing `.sql` migration files

**Returns:**
- `*RunResult` - Execution outcome containing count and list of applied migrations
- `error` - Error if migration failed (automatically rolls back the transaction)

**Example:**
```go
result, err := migration.RunDB(ctx, db, "./migrations")
if err != nil {
    log.Fatalf("Migration failed: %v", err)
}
fmt.Printf("Applied %d migrations\n", result.Applied)
```

#### `StatusDB(ctx context.Context, db *sql.DB, migrationsDir string) (*StatusResult, error)`

Inspects migration status (applied vs pending) by comparing `.sql` files against the live database `schema_migrations` table.

**Example:**
```go
status, err := migration.StatusDB(ctx, db, "./migrations")
if err != nil {
    log.Fatal(err)
}
for _, s := range status.Migrations {
    fmt.Printf("[%t] %s - %s\n", s.Applied, s.Version, s.Name)
}
```

#### `NewMigrator(migrationsDir string) *Migrator`

Creates a new Migrator instance.

**Parameters:**
- `migrationsDir` - Path to the migrations directory

**Returns:**
- `*Migrator` - Migrator instance (nil if creation failed)

**Example:**
```go
m := migration.NewMigrator("./migrations")
if m == nil {
    log.Fatal("Failed to create migrator")
}
defer m.Close()
```

#### `(*Migrator) Close()`

Releases resources associated with the migrator.

**Example:**
```go
m := migration.NewMigrator("./migrations")
defer m.Close()
```

#### `(*Migrator) Run() (*RunResult, error)`

Applies all pending migrations.

**Returns:**
- `*RunResult` - Result of the operation
- `error` - Error if the operation failed

**Example:**
```go
result, err := m.Run()
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Applied %d migrations\n", result.Applied)
```

#### `(*Migrator) Status() (*StatusResult, error)`

Returns the status of all migrations.

**Returns:**
- `*StatusResult` - Status of all migrations
- `error` - Error if the operation failed

**Example:**
```go
status, err := m.Status()
if err != nil {
    log.Fatal(err)
}
for _, s := range status.Migrations {
    fmt.Printf("[%s] %s - %s\n", s.Applied, s.Version, s.Name)
}
```

#### `(*Migrator) Create(name, content string) (*CreateResult, error)`

Creates a new migration file.

**Parameters:**
- `name` - Human-readable name for the migration
- `content` - SQL content of the migration

**Returns:**
- `*CreateResult` - Result of the operation
- `error` - Error if the operation failed

**Example:**
```go
result, err := m.Create("add products table", `
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL
    );
`)
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Created: %s\n", result.Path)
```

#### `(*Migrator) RemovePending() (*RemoveResult, error)`

Removes the last pending migration file.

**Returns:**
- `*RemoveResult` - Result of the operation
- `error` - Error if the operation failed

**Example:**
```go
result, err := m.RemovePending()
if err != nil {
    log.Fatal(err)
}
if result.Removed != "" {
    fmt.Printf("Removed: %s\n", result.Removed)
}
```

#### `(*Migrator) GenerateModels(schemaPath, targetLang, outputDir, pkgName string) (*GenerateModelsResult, error)`

Generates strongly-typed model classes from a database schema (such as DrawDB JSON).

**Parameters:**
- `schemaPath` - Path to schema file (e.g. `"schema/drawdb.json"`)
- `targetLang` - Target language (`"go"`, `"node"`, `"python"`, `"csharp"`). Defaults to `"go"`.
- `outputDir` - Destination folder (e.g. `"./internal/models"`)
- `pkgName` - Package or namespace name (e.g. `"models"`)

**Returns:**
- `*GenerateModelsResult` - Result containing count and generated file paths
- `error` - Error if generation failed

**Example:**
```go
result, err := m.GenerateModels("schema/drawdb.json", "go", "./internal/models", "models")
if err != nil {
    log.Fatal(err)
}
fmt.Printf("Generated %d models in %s\n", result.Count, "./internal/models")
```

## Examples

### Basic Migration Workflow

```go
package main

import (
    "fmt"
    "log"
    "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    // Check current status
    status, err := m.Status()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Current migration status:")
    for _, s := range status.Migrations {
        statusStr := "PENDING"
        if s.Applied {
            statusStr = "APPLIED"
        }
        fmt.Printf("  [%s] %s - %s\n", statusStr, s.Version, s.Name)
    }

    // Run pending migrations
    result, err := m.Run()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("\nApplied %d migrations:\n", result.Applied)
    for _, mig := range result.Migrations {
        fmt.Printf("  - %s: %s\n", mig.Version, mig.Name)
    }
}
```

### Creating a New Migration

```go
result, err := m.Create("add employees table", `
    CREATE TABLE IF NOT EXISTS m_employees (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES m_users(id),
        branch_id UUID REFERENCES m_branches(id),
        employee_code VARCHAR(20) NOT NULL UNIQUE,
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        hire_date DATE NOT NULL,
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_employees_user ON m_employees(user_id);
    CREATE INDEX IF NOT EXISTS idx_employees_branch ON m_employees(branch_id);
    CREATE INDEX IF NOT EXISTS idx_employees_code ON m_employees(employee_code);
`)
```

## Integration with Web Frameworks

### Gin Framework (Startup Auto-Migration)

```go
package main

import (
    "context"
    "database/sql"
    "log"

    "github.com/gin-gonic/gin"
    _ "github.com/lib/pq"
    migration "github.com/MuchammadTedyA/migradb/go"
)

var db *sql.DB

func main() {
    var err error
    db, err = sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
    if err != nil {
        log.Fatalf("Failed to connect to database: %v", err)
    }
    defer db.Close()

    // Run migrations automatically on server startup
    ctx := context.Background()
    result, err := migration.RunDB(ctx, db, "./migrations")
    if err != nil {
        log.Fatalf("Failed to run migrations on startup: %v", err)
    }
    log.Printf("Startup migrations applied: %d", result.Applied)

    // Setup routes
    r := gin.Default()
    r.GET("/migrations/status", migrationStatusHandler)
    r.POST("/migrations/run", migrationRunHandler)

    r.Run(":8080")
}

func migrationStatusHandler(c *gin.Context) {
    status, err := migration.StatusDB(c.Request.Context(), db, "./migrations")
    if err != nil {
        c.JSON(500, gin.H{"error": err.Error()})
        return
    }
    c.JSON(200, status)
}

func migrationRunHandler(c *gin.Context) {
    result, err := migration.RunDB(c.Request.Context(), db, "./migrations")
    if err != nil {
        c.JSON(500, gin.H{"error": err.Error()})
        return
    }
    c.JSON(200, result)
}
```

### Fiber Framework

```go
package main

import (
    "context"
    "database/sql"
    "log"

    "github.com/gofiber/fiber/v2"
    _ "github.com/mattn/go-sqlite3"
    migration "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    db, err := sql.Open("sqlite3", "./app.db")
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    // Auto-migrate on start
    ctx := context.Background()
    if _, err := migration.RunDB(ctx, db, "./migrations"); err != nil {
        log.Fatalf("Migration failed: %v", err)
    }

    app := fiber.New()

    app.Get("/migrations/status", func(c *fiber.Ctx) error {
        status, err := migration.StatusDB(c.Context(), db, "./migrations")
        if err != nil {
            return c.Status(500).JSON(fiber.Map{"error": err.Error()})
        }
        return c.JSON(status)
    })

    app.Listen(":8080")
}
```

## Database Connection Drivers

`migration.RunDB` and `migration.StatusDB` operate on standard Go `*sql.DB` connections:

### PostgreSQL (`lib/pq` or `pgx/stdlib`)

```go
import (
    "context"
    "database/sql"
    _ "github.com/lib/pq"
    migration "github.com/MuchammadTedyA/migradb/go"
)

db, err := sql.Open("postgres", "postgres://user:password@localhost:5432/dbname?sslmode=disable")
res, err := migration.RunDB(context.Background(), db, "./migrations")
```

### MySQL (`go-sql-driver/mysql`)

```go
import (
    "context"
    "database/sql"
    _ "github.com/go-sql-driver/mysql"
    migration "github.com/MuchammadTedyA/migradb/go"
)

db, err := sql.Open("mysql", "user:password@tcp(127.0.0.1:3306)/dbname?multiStatements=true")
res, err := migration.RunDB(context.Background(), db, "./migrations")
```

### SQLite (`mattn/go-sqlite3` or `modernc.org/sqlite`)

```go
import (
    "context"
    "database/sql"
    _ "github.com/mattn/go-sqlite3"
    migration "github.com/MuchammadTedyA/migradb/go"
)

db, err := sql.Open("sqlite3", "./app.db")
res, err := migration.RunDB(context.Background(), db, "./migrations")
```
