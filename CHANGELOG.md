# Changelog

All notable changes to MigrDB will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-09-20

### Added
- **Universal Runtime Auto-Migration & Schema Sync Engine (Approach 1)**:
  - Synchronizes database schemas directly from visual DrawDB diagrams (`drawdb.json`) at runtime without manual SQL migration authoring.
  - Multi-dialect support: default **PostgreSQL**, with full **MySQL** and **SQLite** support.
  - Zero migration regeneration required when switching databases between development and production.
  - Snapshot persistence across `<schema_dir>/drawdb_snapshot.json` and `<migrations_dir>/.schema_snapshot.json`.
  - Default project root folders `./migrations` and `./schema`, with support for custom directory overrides.
  - Cross-language runtime support: Go (`SyncDB`), Node.js (`syncDatabase`), Python (`sync_database`), and CLI (`migradb diff`).

### Fixed
- Fixed snapshot resolution in `find_snapshot` to strictly inspect caller-specified migration and schema directories.
- Added `schema_dir` support to Rust FFI, Go Zero-CGO Windows DLL, and CGO bindings.

## [0.1.1] - 2026-09-20

### Added
- **Standalone CLI tool (`migradb`)** with subcommands for `init`, `create`, `status`, and `generate`.
- **Node.js CLI via `npx migradb`** bundled directly into the `migradb` npm package (`bin/migradb.js`).
- **DrawDB Model Generator Exposed across all bindings**:
  - Node.js: `generateModels(schemaPath, targetLang, outputDir, pkg)` and `migrator.generateModels(...)`.
  - Python: `generate_models(schema_path, target_lang, output_dir, pkg)` and `migrator.generate_models(...)`.
  - Go: `m.GenerateModels(...)`.
- **Rust model generator (`TargetLanguage::Rust`)** creating serde-compatible models and `sqlx::FromRow` derivations.
- **SQL DDL generator (`TargetLanguage::Sql`)** creating schema tables, constraints, and foreign keys for PostgreSQL, MySQL, and SQLite.
- **Direct Database Execution Helpers**:
  - Go: `RunDB` and `StatusDB` for transaction-safe migrations using Go standard library `database/sql`.
  - Node.js: `runOnDatabase` and `statusOnDatabase` supporting `pg`, `mysql2`, and `better-sqlite3`.
  - Python: `run_on_connection` and `status_on_connection` supporting standard Python DB-API 2.0 connections (`psycopg2`, `sqlite3`, `pymysql`).
- Comprehensive documentation, guides, and practical database examples for Node.js and Python.

### Fixed
- Fixed PyO3 compilation error in Python bindings by deriving `Clone` on `TargetLanguage`.
- Decoupled GitHub Actions `publish-npm` and `publish-pypi` into dedicated single-runner jobs to prevent race conditions during multi-platform matrix builds.

## [0.1.0] - 2026-09-12

### Added
- Initial release of MigrDB (v0.1.0)
- High-performance Rust core migration engine (`migration-engine`)
- Windows dynamic DLL loading without CGO using `syscall.NewLazyDLL`
- Ecosystem-aware Model Class Generator (Entity Developer / EF Core style):
  - Go struct models with `json` and `db` tags, nullable pointers, and navigation properties
  - Node.js / TypeScript interfaces and classes with `Date`, `number`, and barrel `index.ts`
  - Python SQLAlchemy 2.0 declarative models (`Mapped[...]`, `mapped_column`, `relationship()`)
  - C# EF Core entity classes (`[Table]`, `[Key]`, `[ForeignKey]`, `[InverseProperty]`) & `MigraDbContext`
  - DrawDB JSON schema parser with automatic 1-to-many relationship mapping
  - Automatic table prefix stripping (`m_`, `t_`, `sys_`, `map_`) and table name singularization
- Go bindings (`github.com/MuchammadTedyA/migradb/go`) via dynamic loading (Windows) and CGO (Linux/macOS) with fallback stubs
- Node.js bindings (`migradb`) via napi-rs
- Python bindings (`migradb`) via PyO3
- Timestamp-based versioning (`YYYYMMDDHHmmss_slug.sql`)
- Immutable migration history and roll-forward architecture
- Transactional migration execution
- Schema version tracking via `schema_migrations`
- Multi-database support (PostgreSQL, MySQL, SQLite)
- Full documentation suite with API references, guides, and cross-platform examples

### Rust Core
- Migration file parser (`parser.rs`)
- In-memory and SQL version trackers (`tracker.rs`)
- Migrator with builder pattern (`migrator.rs`)
- Model codegen engine (`codegen::generate_models`)
- C FFI bindings (`migrator_new`, `migrator_run`, `migrator_status`, `migrator_create`, `migrator_remove_pending`, `migrator_generate_models`, `migrator_free`)

### Go Package
- `migration.NewMigrator()` constructor
- `Run()`, `Status()`, `Create()`, `RemovePending()`, `GenerateModels()` methods
- Windows dynamic DLL loading without CGO
- Fallback stubs for non-CGO compilation
- Automatic resource cleanup

### Node.js Package
- `migradb` npm package
- TypeScript definitions included (`index.d.ts`)
- Pre-built binaries for major platforms
- Express, Fastify, NestJS integration examples

### Python Package
- `migradb` PyPI package
- Type hints support
- Flask, FastAPI, Django integration examples
- Async support via `asyncio.to_thread`

[Unreleased]: https://github.com/MuchammadTedyA/migradb/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/MuchammadTedyA/migradb/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MuchammadTedyA/migradb/releases/tag/v0.1.0
