# MigrDB Documentation

A high-performance SQL database migration and schema modeling library with a Rust core engine, providing native bindings for Go, Node.js (npm), and Python, plus an ecosystem-aware model generator.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Core Components](#core-components)
- [Features](#features)
- [Installation](#installation)
- [Documentation Index](#documentation-index)
- [Design Principles](#design-principles)
- [Contributing](#contributing)
- [License](#license)

## Overview

MigrDB provides a unified, cross-platform solution for managing SQL database migrations and generating strongly typed domain models from visual database schemas. Built with a Rust core for maximum performance and reliability, it offers native bindings for Go, Node.js, and Python environments.

The library adheres to timestamp-based versioning (`YYYYMMDDHHmmss_slug.sql`), immutable migration history, and strict transactional safety.

## Quick Start

### Go

```go
package main

import (
    "fmt"
    "log"

    "github.com/centra/migration"
)

func main() {
    // 1. Initialize migrator (uses Zero-CGO dynamic DLL loading on Windows)
    m := migration.NewMigrator("./migrations")
    if m == nil {
        log.Fatal("Failed to initialize migrator")
    }
    defer m.Close()

    // 2. Apply all pending migrations
    result, err := m.Run()
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("Applied %d migrations\n", result.Applied)

    // 3. Generate Go model structs from DrawDB schema
    modelRes, err := m.GenerateModels("schema/drawdb.json", "go", "./internal/models", "models")
    if err != nil {
        log.Fatalf("Model generation failed: %v", err)
    }
    fmt.Printf("Generated %d Go model files\n", modelRes.Count)
}
```

### Node.js

```javascript
const { Migrator } = require('@centra/migradb');

const m = new Migrator('./migrations');
const result = m.run();

if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
} else {
    console.error(`Migration error: ${result.error}`);
}
```

### Python

```python
from migradb import Migrator

m = Migrator("./migrations")
result = m.run()

if result.success:
    print(f"Applied {result.applied} migrations")
else:
    print(f"Migration error: {result.error}")
```

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

## Core Components

| Component | Description |
|-----------|-------------|
| **Parser** | Reads, validates, and orders SQL migration files using `YYYYMMDDHHmmss_slug.sql` |
| **Tracker** | Tracks executed migrations in the `schema_migrations` table with UTC execution timestamps |
| **Migrator** | Orchestrates transactional migration execution with builder pattern and status queries |
| **Codegen** | Parses DrawDB visual schema diagrams, strips table prefixes (`m_`, `t_`, `sys_`, `map_`), singularizes names, and generates typed models for Go, TypeScript, Python (SQLAlchemy 2), and C# (EF Core) |
| **FFI Layer** | High-performance C ABI interface (`migrator_*`) for cross-language interoperability |

## Features

- **Timestamp-based versioning** - Strict chronological ordering preventing branch conflicts
- **Transactional safety** - Every migration runs in an isolated atomic transaction
- **Immutable history** - Roll-forward architecture ensures reproducible deployments
- **Idempotent SQL** - Native support for `IF NOT EXISTS` / `IF EXISTS` schema guards
- **Ecosystem-aware Model Generation** - Generates native ORM/struct models from visual schemas
- **Multi-language support** - Bindings for Go, Node.js, and Python from a single Rust core
- **Zero-CGO on Windows** - Dynamic DLL loading via `syscall.NewLazyDLL` without GCC/MinGW
- **Schema tracking** - Automatic `schema_migrations` audit table management
- **Cross-platform** - Tested on Windows, Linux, and macOS

## Installation

### Pre-built Packages

```bash
# Go
go get github.com/centra/migration

# Node.js
npm install @centra/migradb

# Python
pip install migradb
```

### Building from Source

```bash
# Automated builds
.\build.ps1   # Windows PowerShell
./build.sh    # Unix / Linux / macOS

# Rust Core manually
cargo build --release
```

## Documentation Index

### Guides

- [Migration Guide](guides/migrations.md) - Creating migrations, table naming conventions, and DrawDB modeling
- [Go Guide](guides/go.md) - Go integration, Windows dynamic DLL loading, and code generation
- [Node.js Guide](guides/nodejs.md) - Node.js and TypeScript usage with Express, Fastify, and NestJS
- [Python Guide](guides/python.md) - Python and async usage with FastAPI, Flask, and Django
- [Database Guide](guides/database.md) - PostgreSQL, MySQL, and SQLite configuration and best practices

### API Reference

- [Go API Reference](api/go.md) - Structs, methods (`Run`, `Status`, `Create`, `GenerateModels`), and DLL resolution
- [Node.js API Reference](api/nodejs.md) - `@centra/migradb` class and interface documentation
- [Python API Reference](api/python.md) - `migradb` class and type hint documentation

### Examples

- [Go Examples](examples/go.md) - Migration runner, web frameworks, and DrawDB model generation
- [Node.js Examples](examples/nodejs.md) - Express, Fastify, and migration management
- [Python Examples](examples/python.md) - FastAPI, Flask, and async workers

## Design Principles

1. **Immutable History** - Never edit old migrations; always create new files to modify schema.
2. **Idempotent SQL** - Protect schema definitions with `IF NOT EXISTS` / `IF EXISTS` guards.
3. **Transactional Safety** - Each migration executes within an isolated database transaction.
4. **Version Tracking** - Simple timestamp-based versioning eliminates version conflicts.
5. **Roll-Forward Architecture** - Fix schema errors by rolling forward rather than running destructive down migrations.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](../CONTRIBUTING.md) for setup and guidelines.

## License

MIT License - see [LICENSE](../LICENSE) for details.
