# Go Examples

Practical examples for using the MigrDB in Go projects.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Web Server Integration](#web-server-integration)
- [CLI Tool](#cli-tool)
- [Multi-Database](#multi-database)
- [Custom Migration Generator](#custom-migration-generator)
- [Model Generation from DrawDB](#model-generation-from-drawdb)

## Basic Usage

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

    // Check status
    status, err := m.Status()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println("Migration Status:")
    for _, s := range status.Migrations {
        statusStr := "PENDING"
        if s.Applied {
            statusStr = "APPLIED"
        }
        fmt.Printf("  [%s] %s - %s\n", statusStr, s.Version, s.Name)
    }

    // Run migrations
    result, err := m.Run()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("\nApplied %d migrations\n", result.Applied)
}
```

## Web Server Integration

### Gin Framework

```go
package main

import (
    "net/http"

    "github.com/MuchammadTedyA/migradb/go"
    "github.com/gin-gonic/gin"
)

var migrator *migration.Migrator

func main() {
    migrator = migration.NewMigrator("./migrations")
    defer migrator.Close()

    // Run migrations on startup
    if _, err := migrator.Run(); err != nil {
        log.Fatal("Migration failed:", err)
    }

    r := gin.Default()

    r.GET("/api/migrations/status", func(c *gin.Context) {
        status, err := migrator.Status()
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
            return
        }
        c.JSON(http.StatusOK, status)
    })

    r.POST("/api/migrations/run", func(c *gin.Context) {
        result, err := migrator.Run()
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
            return
        }
        c.JSON(http.StatusOK, result)
    })

    r.Run(":8080")
}
```

### Fiber Framework

```go
package main

import (
    "github.com/MuchammadTedyA/migradb/go"
    "github.com/gofiber/fiber/v2"
)

func main() {
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    app := fiber.New()

    app.Get("/api/migrations/status", func(c *fiber.Ctx) error {
        status, err := m.Status()
        if err != nil {
            return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
                "error": err.Error(),
            })
        }
        return c.JSON(status)
    })

    app.Post("/api/migrations/run", func(c *fiber.Ctx) error {
        result, err := m.Run()
        if err != nil {
            return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
                "error": err.Error(),
            })
        }
        return c.JSON(result)
    })

    app.Listen(":8080")
}
```

## CLI Tool

```go
package main

import (
    "fmt"
    "os"

    "github.com/MuchammadTedyA/migradb/go"
)

func main() {
    if len(os.Args) < 2 {
        printUsage()
        return
    }

    migrationsDir := "./migrations"
    command := os.Args[1]

    if len(os.Args) > 2 {
        migrationsDir = os.Args[2]
    }

    m := migration.NewMigrator(migrationsDir)
    defer m.Close()

    switch command {
    case "run":
        handleRun(m)
    case "status":
        handleStatus(m)
    case "create":
        handleCreate(m)
    case "remove":
        handleRemove(m)
    default:
        fmt.Fprintf(os.Stderr, "Unknown command: %s\n", command)
        printUsage()
        os.Exit(1)
    }
}

func handleRun(m *migration.Migrator) {
    result, err := m.Run()
    if err != nil {
        fmt.Fprintln(os.Stderr, "Error:", err)
        os.Exit(1)
    }

    if result.Applied == 0 {
        fmt.Println("No pending migrations")
    } else {
        fmt.Printf("Applied %d migrations:\n", result.Applied)
        for _, mig := range result.Migrations {
            fmt.Printf("  - %s: %s\n", mig.Version, mig.Name)
        }
    }
}

func handleStatus(m *migration.Migrator) {
    status, err := m.Status()
    if err != nil {
        fmt.Fprintln(os.Stderr, "Error:", err)
        os.Exit(1)
    }

    for _, s := range status.Migrations {
        statusStr := "PENDING"
        if s.Applied {
            statusStr = "APPLIED"
        }
        fmt.Printf("[%s] %s - %s\n", statusStr, s.Version, s.Name)
    }
}

func handleCreate(m *migration.Migrator) {
    if len(os.Args) < 3 {
        fmt.Fprintln(os.Stderr, "Usage: migrate create <name>")
        os.Exit(1)
    }

    name := os.Args[3]
    result, err := m.Create(name, "-- Add your SQL here\n")
    if err != nil {
        fmt.Fprintln(os.Stderr, "Error:", err)
        os.Exit(1)
    }

    fmt.Printf("Created: %s\n", result.Path)
}

func handleRemove(m *migration.Migrator) {
    result, err := m.RemovePending()
    if err != nil {
        fmt.Fprintln(os.Stderr, "Error:", err)
        os.Exit(1)
    }

    if result.Removed != "" {
        fmt.Printf("Removed: %s\n", result.Removed)
    } else {
        fmt.Println(result.Message)
    }
}

func printUsage() {
    fmt.Println("Usage: migrate <command> [directory]")
    fmt.Println("")
    fmt.Println("Commands:")
    fmt.Println("  run      Apply pending migrations")
    fmt.Println("  status   Show migration status")
    fmt.Println("  create   Create new migration file")
    fmt.Println("  remove   Remove last pending migration")
}
```

## Multi-Database

```go
package main

import (
    "fmt"
    "log"
    "sync"

    "github.com/MuchammadTedyA/migradb/go"
)

type DatabaseMigrator struct {
    name      string
    migrator  *migration.Migrator
}

func main() {
    databases := []DatabaseMigrator{
        {name: "users", migrator: migration.NewMigrator("./migrations/users")},
        {name: "orders", migrator: migration.NewMigrator("./migrations/orders")},
        {name: "products", migrator: migration.NewMigrator("./migrations/products")},
    }

    defer func() {
        for _, db := range databases {
            db.migrator.Close()
        }
    }()

    var wg sync.WaitGroup
    errors := make(chan error, len(databases))

    for _, db := range databases {
        wg.Add(1)
        go func(db DatabaseMigrator) {
            defer wg.Done()
            result, err := db.migrator.Run()
            if err != nil {
                errors <- fmt.Errorf("%s: %w", db.name, err)
                return
            }
            fmt.Printf("%s: Applied %d migrations\n", db.name, result.Applied)
        }(db)
    }

    wg.Wait()
    close(errors)

    for err := range errors {
        log.Println("Error:", err)
    }
}
```

## Custom Migration Generator

```go
package main

import (
    "fmt"
    "strings"

    "github.com/MuchammadTedyA/migradb/go"
)

type TableColumn struct {
    Name       string
    Type       string
    Nullable   bool
    Default    string
    PrimaryKey bool
    Unique     bool
}

type TableMigration struct {
    migrator *migration.Migrator
}

func NewTableMigration(m *migration.Migrator) *TableMigration {
    return &TableMigration{migrator: m}
}

func (t *TableMigration) CreateTable(name string, columns []TableColumn) error {
    var columnDefs []string
    var indexes []string

    for _, col := range columns {
        def := fmt.Sprintf("%s %s", col.Name, col.Type)

        if !col.Nullable {
            def += " NOT NULL"
        }
        if col.Default != "" {
            def += fmt.Sprintf(" DEFAULT %s", col.Default)
        }
        if col.PrimaryKey {
            def += " PRIMARY KEY"
        }
        if col.Unique {
            def += " UNIQUE"
        }

        columnDefs = append(columnDefs, def)

        if !col.PrimaryKey {
            indexes = append(indexes, fmt.Sprintf(
                "CREATE INDEX IF NOT EXISTS idx_%s_%s ON %s(%s);",
                name, col.Name, name, col.Name,
            ))
        }
    }

    sql := fmt.Sprintf("CREATE TABLE IF NOT EXISTS %s (\n    %s\n);\n\n%s",
        name,
        strings.Join(columnDefs, ",\n    "),
        strings.Join(indexes, "\n"),
    )

    result, err := t.migrator.Create(fmt.Sprintf("create %s table", name), sql)
    if err != nil {
        return err
    }

    fmt.Printf("Created: %s\n", result.Path)
    return nil
}

func main() {
    m := migration.NewMigrator("./migrations")
    defer m.Close()

    tm := NewTableMigration(m)

    err := tm.CreateTable("m_users", []TableColumn{
        {Name: "id", Type: "UUID", PrimaryKey: true, Default: "gen_random_uuid()"},
        {Name: "username", Type: "VARCHAR(100)", Unique: true},
        {Name: "email", Type: "VARCHAR(255)", Unique: true},
        {Name: "password_hash", Type: "TEXT"},
        {Name: "is_active", Type: "BOOLEAN", Default: "true"},
        {Name: "created_at", Type: "TIMESTAMPTZ", Default: "NOW()"},
    })
    if err != nil {
        panic(err)
    }
}
```

## Model Generation from DrawDB

Generates strongly-typed Go model structs (with `json` and `db` tags, nullable pointers, and navigation relation properties) directly from DrawDB JSON:

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

    // Generate Go models into ./internal/models under package "models"
    result, err := m.GenerateModels(
        "schema/drawdb.json", // Path to DrawDB JSON export
        "go",                 // Target language: go, node, python, or csharp
        "./internal/models",  // Output directory
        "models",             // Package name
    )
    if err != nil {
        log.Fatalf("Model generation failed: %v", err)
    }

    fmt.Printf("Successfully generated %d Go model files!\n", result.Count)
    for _, file := range result.Files {
        fmt.Printf("  - %s\n", file)
    }
}
```

### Sample Generated Model (`internal/models/branch.go`):

```go
// Code generated by MigraDB. DO NOT EDIT.

package models

import (
    "time"
    "github.com/google/uuid"
)

// Branch maps to database table `m_branches`.
type Branch struct {
    ID        uuid.UUID  `json:"id" db:"id"`
    CompanyID uuid.UUID  `json:"company_id" db:"company_id"`
    Code      string     `json:"code" db:"code"`
    Name      string     `json:"name" db:"name"`
    CreatedAt time.Time  `json:"created_at" db:"created_at"`

    // Navigation Properties
    Company   *Company   `json:"company,omitempty" db:"-"`
    UserRoles []UserRole `json:"user_roles,omitempty" db:"-"`
}
```

