# Contributing Guide

Thank you for your interest in contributing to MigrDB!

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Building](#building)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/MuchammadTedyA/migradb.git`
3. Create a branch: `git checkout -b feature/your-feature`

## Development Setup

### Rust

Install Rust from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Go

Install Go 1.21+ from [golang.org](https://golang.org/).

### Node.js

Install Node.js 18+ and npm from [nodejs.org](https://nodejs.org/).

### Python

Install Python 3.8+ from [python.org](https://python.org/).

## Project Structure

```
migradb/
├── src/                      # Rust core migration & codegen engine
│   ├── lib.rs                # Library entry point and re-exports
│   ├── error.rs              # Typed error definitions (MigrationError)
│   ├── parser.rs             # SQL migration file parser & validator
│   ├── tracker.rs            # Schema migration history trackers
│   ├── migrator.rs           # Migrator coordinator & builder pattern
│   ├── ffi.rs                # C-compatible FFI export functions
│   └── codegen/              # Ecosystem-aware Model Class Generator
│       ├── mod.rs            # TargetLanguage enum & generate_models entrypoint
│       ├── drawdb_parser.rs  # DrawDB schema parser & relation detection
│       ├── naming.rs         # Prefix stripping & singularization
│       ├── schema.rs         # Abstract schema representation
│       ├── golang.rs         # Go struct code generation
│       ├── node_ts.rs        # TypeScript interface & barrel generation
│       ├── python.rs         # SQLAlchemy 2.0 declarative code generation
│       └── csharp.rs         # EF Core entity & DbContext generation
├── go/                       # Go language bindings (github.com/MuchammadTedyA/migradb/go)
│   ├── go.mod                # Go module definition
│   ├── types.go              # Shared Go types and result structs
│   ├── migrator_windows.go   # Zero-CGO Windows dynamic DLL loading
│   ├── migrator_cgo.go       # CGO bindings for Linux and macOS
│   ├── migrator_nocgo.go     # Fallback stubs when compiled without CGO
│   ├── migrator_test.go      # Go integration & codegen tests
│   └── example/              # Example Go application
├── npm/                      # Node.js bindings (migradb)
│   ├── src/lib.rs            # napi-rs Rust wrapper
│   ├── index.js              # Node.js entrypoint
│   ├── index.d.ts            # TypeScript definitions
│   └── package.json          # npm package manifest
├── python/                   # Python bindings (migradb)
│   ├── src/lib.rs            # PyO3 Rust extension module
│   ├── __init__.py           # Python module exports
│   └── pyproject.toml        # Maturin build configuration
├── migrations/               # Example migration files
├── docs/                     # Documentation suite
├── build.ps1                 # Windows automated build script
├── build.sh                  # Unix/macOS automated build script
├── Cargo.toml                # Root Cargo workspace manifest
├── CHANGELOG.md              # Project change log
└── README.md                 # Project overview and quick start
```

## Building

### Automated Build

```bash
# Windows PowerShell
.\build.ps1

# Unix / macOS
./build.sh
```

### Build Individual Components

```bash
# Rust core engine
cargo build --release

# Go bindings
cd go && go build ./...

# Node.js bindings
cd npm && npm install && npm run build

# Python bindings
cd python && pip install maturin && maturin develop
```

## Testing

### Rust Core Tests

Runs unit and integration tests for migration parser, trackers, and the model codegen engine:

```bash
cargo test
```

### Go Tests

```bash
cd go && go test -v ./...
```

### Node.js Tests

```bash
cd npm && npm test
```

### Python Tests

```bash
cd python && python -m pytest
```

## Code Style

### Rust

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/)
- Run `cargo fmt` before committing
- Run `cargo clippy` for linting

### Go

- Follow [Effective Go](https://golang.org/doc/effective_go)
- Run `gofmt -w .` before committing
- Run `go vet ./...` for linting

### JavaScript / TypeScript

- Follow [StandardJS](https://standardjs.com/) style
- Run `npx standard --fix` before committing

### Python

- Follow [PEP 8](https://peps.python.org/pep-0008/)
- Run `black .` before committing
- Run `flake8` for linting

## Submitting Changes

1. Ensure all tests pass (`cargo test`, `go test ./...`)
2. Update documentation if introducing new features or APIs
3. Add an entry to `CHANGELOG.md` under `[Unreleased]`
4. Create a pull request

## Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new migration command
fix: resolve parser error for empty files
docs: update API reference
test: add integration tests
```

## Release Process

1. Update version in `Cargo.toml`, `npm/package.json`, and `python/pyproject.toml`
2. Update `CHANGELOG.md` with release notes and date
3. Create a git tag: `git tag v0.1.0`
4. Push the tag: `git push origin v0.1.0`
5. CI/CD will build and publish packages

## Code of Conduct

Be respectful and constructive in all interactions.
