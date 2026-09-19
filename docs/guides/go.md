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
go mod edit -replace github.com/centra/migration=../migradb/go
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

```go
package main

import (
    "fmt"
    "log"
    "github.com/centra/migration"
)

func main() {
    // Create migrator instance
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    // Run all pending migrations
    result, err := m.Run()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Applied %d migrations\n", result.Applied)

    // Generate Go model classes from DrawDB schema
    modelRes, err := m.GenerateModels("schema/drawdb.json", "go", "./internal/models", "models")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Generated %d model classes in ./internal/models\n", modelRes.Count)
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
    "github.com/centra/migration"
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

### Gin Framework

```go
package main

import (
    "github.com/gin-gonic/gin"
    "github.com/centra/migration"
)

var migrator *migration.Migrator

func main() {
    // Initialize migrator
    migrator = migration.NewMigrator("./migrations")
    defer migrator.Close()

    // Run migrations on startup
    if _, err := migrator.Run(); err != nil {
        log.Fatal("Failed to run migrations:", err)
    }

    // Setup routes
    r := gin.Default()
    r.GET("/migrations/status", migrationStatusHandler)
    r.POST("/migrations/run", migrationRunHandler)

    r.Run(":8080")
}

func migrationStatusHandler(c *gin.Context) {
    status, err := migrator.Status()
    if err != nil {
        c.JSON(500, gin.H{"error": err.Error()})
        return
    }
    c.JSON(200, status)
}

func migrationRunHandler(c *gin.Context) {
    result, err := migrator.Run()
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
    "github.com/gofiber/fiber/v2"
    "github.com/centra/migration"
)

func main() {
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    app := fiber.New()

    app.Get("/migrations/status", func(c *fiber.Ctx) error {
        status, err := m.Status()
        if err != nil {
            return c.Status(500).JSON(fiber.Map{"error": err.Error()})
        }
        return c.JSON(status)
    })

    app.Listen(":8080")
}
```

## Database Connection

The migration library handles file parsing and version tracking. For actual database execution, you need to integrate with your database driver.

### PostgreSQL with pgx

```go
package main

import (
    "context"
    "github.com/jackc/pgx/v5/pgxpool"
    "github.com/centra/migration"
)

func main() {
    ctx := context.Background()

    // Connect to database
    db, err := pgxpool.New(ctx, "postgres://user:pass@localhost/dbname")
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
