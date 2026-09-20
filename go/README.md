# MigraDB Go Package (`github.com/MuchammadTedyA/migradb/go`)

High-performance SQL database migrations and DrawDB model generation for Go, powered by a compiled Rust core engine.

## Key Features

- ⚡ **High-Performance Rust Core Engine**: Ultra-fast file parsing, validation, and sorting.
- 🔄 **Universal Schema Sync (Approach 1)**: Sync live databases directly from DrawDB (`drawdb.json`) with zero migration regeneration on database switch.
- 🎯 **Multi-Dialect Support**: PostgreSQL by default, with complete native support for MySQL and SQLite.
- 📁 **Auto Directory Conventions**: Auto-creates `./schema` and `./migrations` if missing; allows custom paths.
- 🪟 **Zero-CGO on Windows**: Dynamically loads `migration_engine.dll` without requiring MinGW, GCC, or CGO tooling.
- 🐧 **Native CGO on Linux/macOS**: Direct C-ABI dynamic linking with fallback stubs.
- 🛡️ **Transactional Database Runners (`SyncDB`, `RunDB`)**: 1-line transaction-safe execution for any Go standard library `*sql.DB` driver (PostgreSQL, MySQL, SQLite).
- 📦 **Ecosystem-Aware Model Generation**: Generates clean Go struct files with `json` and `db` tags, pointer types for nullable fields, and 1-to-many relationship navigation.

---

## 1. Installation

```bash
go get github.com/MuchammadTedyA/migradb/go
```

### Shared Library Resolution

- **Windows**: The package searches for `migration_engine.dll` automatically in:
  1. `MIGRATION_ENGINE_DLL` environment variable (if specified)
  2. The application's current working directory
  3. `target/release/migration_engine.dll`
- **Linux / macOS**: Ensure `libmigration_engine.so` or `libmigration_engine.dylib` is in your dynamic library path:
  ```bash
  export LD_LIBRARY_PATH=/path/to/target/release:$LD_LIBRARY_PATH
  ```

---

## 2. Universal Schema Sync (`SyncDB`) - Recommended

Design your schema visually in DrawDB, export the JSON to `./schema/drawdb.json`, and let MigraDB synchronize your database on application startup.

### Why Approach 1?
- **Zero Migration Regeneration**: If you change your database engine midway (e.g. SQLite in local tests -> PostgreSQL in production, or MySQL -> PostgreSQL), **you never need to regenerate migrations**. Simply change your database driver and connection; MigraDB evaluates the schema against the target database and generates the exact DDL statements required for that engine on the fly.
- **Default Directory Conventions**: Automatically creates `./schema` and `./migrations` in the root project if missing. If they already exist, files are placed directly inside.
- **Dialect Defaults**: Defaults to **PostgreSQL**. MySQL and SQLite are fully supported via `opts.Dialect`.
- **Audit Logging**: Automatically writes timestamped audit migrations (`migrations/YYYYMMDDHHmmss_sync_drawdb.sql`) and maintains snapshots (`schema/drawdb_snapshot.json`).

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

	// 1. Connect to your database
	db, err := sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
	if err != nil {
		log.Fatalf("Database connection failed: %v", err)
	}
	defer db.Close()

	// 2. Synchronize schema directly from drawdb.json
	res, err := migration.SyncDB(ctx, db, migration.SyncOptions{
		SchemaPath:        "./schema/drawdb.json",     // Default schema file
		MigrationsDir:     "./migrations",            // Auto-created if missing
		Dialect:           migration.DialectPostgres, // "postgres" (default), "mysql", or "sqlite"
		SaveMigrationFile: true,                      // Save audit .sql migration
		MigrationName:     "init_schema",             // Migration slug
	})
	if err != nil {
		log.Fatalf("Sync failed: %v", err)
	}

	if res.IsEmpty {
		fmt.Println("✅ Schema is already up to date!")
	} else {
		fmt.Printf("✅ Applied %d DDL statement(s)!\n", res.Applied)
		fmt.Printf("   Audit migration: %s\n", res.MigrationPath)
	}
}
```

---

## 3. Quick Start: Manual Migration Files (`RunDB`)


Use `migration.RunDB` to apply migrations directly to your database within an atomic transaction. Works with any `*sql.DB` driver (PostgreSQL, SQLite, MySQL).

### PostgreSQL Example (`lib/pq` or `pgx/stdlib`)

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

	// 1. Connect to your database
	db, err := sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
	if err != nil {
		log.Fatalf("Database connection failed: %v", err)
	}
	defer db.Close()

	// 2. Check migration status
	status, err := migration.StatusDB(ctx, db, "./migrations")
	if err != nil {
		log.Fatalf("Failed to check status: %v", err)
	}
	fmt.Printf("Total migrations tracked: %d\n", len(status.Migrations))
	for _, m := range status.Migrations {
		state := "PENDING"
		if m.Applied {
			state = "APPLIED"
		}
		fmt.Printf("  [%s] %s - %s\n", state, m.Version, m.Name)
	}

	// 3. Apply all pending migrations in atomic transactions
	result, err := migration.RunDB(ctx, db, "./migrations")
	if err != nil {
		log.Fatalf("Migration failed: %v", err)
	}

	fmt.Printf("Successfully applied %d migration(s)!\n", result.Applied)
}
```

### SQLite Example (`mattn/go-sqlite3` or `modernc.org/sqlite`)

```go
package main

import (
	"context"
	"database/sql"
	"fmt"
	"log"

	_ "github.com/mattn/go-sqlite3"
	migration "github.com/MuchammadTedyA/migradb/go"
)

func main() {
	ctx := context.Background()

	db, err := sql.Open("sqlite3", "./app.db")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	result, err := migration.RunDB(ctx, db, "./migrations")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Applied %d SQLite migrations\n", result.Applied)
}
```

---

## 3. Generate Go Models from DrawDB (`drawdb.json`)

Export your database diagram as a JSON file from [DrawDB](https://drawdb.app) and generate strongly-typed Go structs:

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

	// Generate Go model files into ./internal/models
	result, err := m.GenerateModels(
		"schema/drawdb.json", // Path to DrawDB export
		"go",                 // Target: "go"
		"./internal/models",  // Output directory
		"models",             // Package name
	)
	if err != nil {
		log.Fatalf("Model generation failed: %v", err)
	}

	fmt.Printf("Generated %d Go model files!\n", result.Count)
	for _, file := range result.Files {
		fmt.Printf("  - %s\n", file)
	}
}
```

### Generated Go Struct Example (`company.go`):

```go
// <auto-generated> by MigraDB. DO NOT EDIT. </auto-generated>
package models

import (
	"time"
)

type Company struct {
	ID        int64     `json:"id" db:"id"`
	Name      string    `json:"name" db:"name"`
	CreatedAt time.Time `json:"created_at" db:"created_at"`

	// Navigation relations
	Branches []Branch `json:"branches,omitempty"`
}
```

---

## 4. Migration File Management (`Migrator`)

Use the core `Migrator` struct to create or inspect migration files:

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

	// Create a new timestamped migration file
	createRes, err := m.Create("create_users_table", `
		CREATE TABLE IF NOT EXISTS m_users (
			id SERIAL PRIMARY KEY,
			username VARCHAR(100) NOT NULL,
			created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
		);
	`)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Created: %s\n", createRes.Path)

	// Inspect migration files
	statusRes, err := m.Status()
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Total files: %d\n", len(statusRes.Migrations))
}
```

---

## 5. Web Framework Integration (Gin / Fiber)

### Gin Example: Running Migrations on Server Startup

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

func main() {
	db, err := sql.Open("postgres", "postgres://postgres:secret@localhost:5432/myapp?sslmode=disable")
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	// Run pending migrations on application startup
	ctx := context.Background()
	runRes, err := migration.RunDB(ctx, db, "./migrations")
	if err != nil {
		log.Fatalf("Database migration failed: %v", err)
	}
	log.Printf("Migrations applied on startup: %d", runRes.Applied)

	// Setup routes
	r := gin.Default()
	r.GET("/health", func(c *gin.Context) {
		c.JSON(200, gin.H{"status": "ok"})
	})

	r.Run(":8080")
}
```

---

## API Summary

| Function / Method | Description |
|---|---|
| `RunDB(ctx, db, dir)` | Runs pending migrations against `*sql.DB` inside atomic transactions. |
| `StatusDB(ctx, db, dir)` | Queries migration history from `schema_migrations` audit table. |
| `NewMigrator(dir)` | Initializes core migrator for file management and model generation. |
| `m.Create(name, sql)` | Generates a new timestamped migration file (`YYYYMMDDHHmmss_name.sql`). |
| `m.Status()` | Lists all migration files and their state according to the tracker. |
| `m.GenerateModels(schema, lang, out, pkg)` | Generates strongly-typed models from DrawDB JSON diagram. |
| `m.Close()` | Releases FFI and native resources safely. |

