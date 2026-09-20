# Go API Reference

Complete API reference for the MigrDB Go package (`github.com/MuchammadTedyA/migradb/go`).

## Package: `migration`

```go
import migration "github.com/MuchammadTedyA/migradb/go"
```

## Platform Architecture & Runtime Loading

The Go package features dual-mode execution depending on the host operating system:

| Platform | Loading Mechanism | Requirements |
|----------|-------------------|--------------|
| **Windows** | Dynamic DLL loading (`syscall.NewLazyDLL`) | **Zero-CGO!** No GCC, MinGW, or C compiler required. Runs purely in native Go using `migration_engine.dll`. |
| **Linux / macOS** | Standard CGO bindings (`migrator_cgo.go`) | CGO enabled (`CGO_ENABLED=1`), GCC / Clang, and `libmigration_engine.so` / `libmigration_engine.dylib` on library path. |
| **Non-CGO Unix** | Safe fallback stubs (`migrator_nocgo.go`) | Compiles safely but returns an error informing that CGO is required on Unix. |

### Windows DLL Resolution Order

When running on Windows, `migradb/go` automatically attempts to resolve `migration_engine.dll` in this order:

1. `MIGRATION_ENGINE_DLL` environment variable (if defined)
2. Current working directory: `migration_engine.dll`
3. Relative release build: `../target/release/migration_engine.dll`
4. Up-level repo release: `../../migradb/target/release/migration_engine.dll`

---

## Types

### `Migrator`

The main struct managing migrations and model code generation.

```go
type Migrator struct {
    handle unsafe.Pointer
}
```

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| **Close** | `func (m *Migrator) Close()` | Releases the underlying Rust engine handle and resources |
| **Run** | `func (m *Migrator) Run() (*RunResult, error)` | Applies all pending migrations in order |
| **Status** | `func (m *Migrator) Status() (*StatusResult, error)` | Queries the status of all registered migrations |
| **Create** | `func (m *Migrator) Create(name, content string) (*CreateResult, error)` | Creates a new timestamped migration file |
| **RemovePending** | `func (m *Migrator) RemovePending() (*RemoveResult, error)` | Deletes the last migration file if unapplied |
| **GenerateModels** | `func (m *Migrator) GenerateModels(schemaPath, targetLang, outputDir, pkgName string) (*GenerateModelsResult, error)` | Generates typed model classes from DrawDB JSON |

---

### `Migration`

Represents an applied or registered migration entry.

```go
type Migration struct {
    Version string `json:"version"`
    Name    string `json:"name"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Version` | `string` | 14-digit timestamp version (`YYYYMMDDHHmmss`) |
| `Name` | `string` | Migration slug description |

---

### `MigrationStatus`

Represents the execution state of a migration file.

```go
type MigrationStatus struct {
    Version   string  `json:"version"`
    Name      string  `json:"name"`
    Applied   bool    `json:"applied"`
    AppliedAt *string `json:"applied_at,omitempty"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Version` | `string` | 14-digit timestamp version |
| `Name` | `string` | Migration slug description |
| `Applied` | `bool` | `true` if migration has been executed in database |
| `AppliedAt` | `*string` | ISO 8601 UTC timestamp of execution (`nil` if pending) |

---

### `RunResult`

Return value from `m.Run()`.

```go
type RunResult struct {
    Success    bool        `json:"success"`
    Applied    int         `json:"applied"`
    Migrations []Migration `json:"migrations"`
    Error      string      `json:"error,omitempty"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Success` | `bool` | `true` if all pending migrations executed without errors |
| `Applied` | `int` | Count of newly executed migrations |
| `Migrations` | `[]Migration` | List of newly applied migrations |
| `Error` | `string` | Error message string if failed |

---

### `StatusResult`

Return value from `m.Status()`.

```go
type StatusResult struct {
    Success    bool              `json:"success"`
    Migrations []MigrationStatus `json:"migrations"`
    Error      string            `json:"error,omitempty"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Success` | `bool` | `true` if status query succeeded |
| `Migrations` | `[]MigrationStatus` | Complete list of all migrations with execution flags |
| `Error` | `string` | Error message string if failed |

---

### `CreateResult`

Return value from `m.Create()`.

```go
type CreateResult struct {
    Success bool   `json:"success"`
    Path    string `json:"path"`
    Error   string `json:"error,omitempty"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Success` | `bool` | `true` if the file was created successfully |
| `Path` | `string` | Absolute or relative path to the new migration file |
| `Error` | `string` | Error message string if failed |

---

### `RemoveResult`

Return value from `m.RemovePending()`.

```go
type RemoveResult struct {
    Success bool   `json:"success"`
    Removed string `json:"removed,omitempty"`
    Message string `json:"message,omitempty"`
    Error   string `json:"error,omitempty"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Success` | `bool` | `true` if operation completed |
| `Removed` | `string` | Path of deleted migration file (empty if none) |
| `Message` | `string` | Informational message (e.g., "No pending migrations to remove") |
| `Error` | `string` | Error message (e.g. if the migration was already applied) |

---

### `GenerateModelsResult`

Return value from `m.GenerateModels()`.

```go
type GenerateModelsResult struct {
    Success bool     `json:"success"`
    Count   int      `json:"count"`
    Files   []string `json:"files"`
    Error   string   `json:"error,omitempty"`
}
```

| Field | Type | Description |
|-------|------|-------------|
| `Success` | `bool` | `true` if code generation succeeded |
| `Count` | `int` | Number of files written to disk |
| `Files` | `[]string` | Paths of all generated entity and context files |
| `Error` | `string` | Error message string if failed |

---

### `SyncOptions`

Configuration options for `migration.SyncDB()`.

```go
type SyncOptions struct {
    SchemaPath        string // Path to drawdb.json (default: "./schema/drawdb.json")
    MigrationsDir     string // Migrations directory (default: "./migrations")
    SchemaDir         string // Schema directory for snapshots (default: "./schema")
    Dialect           string // "postgres" (default), "mysql", or "sqlite"
    ForceFull         bool   // Force baseline generation
    SaveMigrationFile bool   // Automatically write timestamped audit .sql migration
    MigrationName     string // Migration slug (default: "sync_drawdb")
}
```

### `SyncResult`

Return value from `migration.SyncDB()`.

```go
type SyncResult struct {
    Success       bool     `json:"success"`
    IsEmpty       bool     `json:"is_empty"`
    Applied       int      `json:"applied"`
    DiffSummary   string   `json:"diff_summary"`
    MigrationPath string   `json:"migration_path,omitempty"`
    Statements    []string `json:"statements,omitempty"`
    Error         string   `json:"error,omitempty"`
}
```

---

## Functions

### `SyncDB`

Synchronizes a live `*sql.DB` database directly against a DrawDB visual schema with dynamic multi-dialect translation and snapshot tracking.

```go
func SyncDB(ctx context.Context, db *sql.DB, opts SyncOptions) (*SyncResult, error)
```

- **Features**:
  - Auto-creates `./schema` and `./migrations` folders if missing.
  - Translates schema diffs to PostgreSQL (`DialectPostgres`), MySQL (`DialectMySQL`), or SQLite (`DialectSQLite`).
  - Switching database engine midway requires **zero migration regeneration**.
  - Atomically applies DDL inside a transaction, records `schema_migrations`, writes an audit `.sql` file, and updates snapshots.

### `RunDB`

Executes all pending migration `.sql` files in chronological order against a live `*sql.DB` database inside atomic transactions.

```go
func RunDB(ctx context.Context, db *sql.DB, migrationsDir string) (*RunResult, error)
```

### `StatusDB`

Inspects applied vs pending migrations on a live `*sql.DB` database.

```go
func StatusDB(ctx context.Context, db *sql.DB, migrationsDir string) (*StatusResult, error)
```

### `NewMigrator`


Creates and initializes a new `Migrator` instance.

```go
func NewMigrator(migrationsDir string) *Migrator
```

**Parameters:**
- `migrationsDir` (`string`): Path to directory containing `.sql` migration files.

**Returns:**
- `*Migrator`: Pointer to Migrator instance, or `nil` if initialization failed.

```go
m := migration.NewMigrator("./migrations")
if m == nil {
    log.Fatal("Could not initialize migrator (check DLL or permissions)")
}
defer m.Close()
```

---

## Detailed Method Reference

### `(*Migrator) GenerateModels`

```go
func (m *Migrator) GenerateModels(schemaPath, targetLang, outputDir, pkgName string) (*GenerateModelsResult, error)
```

Generates strongly typed entity models and database context classes directly from a DrawDB diagram JSON export file.

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `schemaPath` | `string` | (Required) | Path to the exported DrawDB JSON file |
| `targetLang` | `string` | `"go"` | Target language: `"go"`, `"node"` / `"ts"`, `"python"`, `"csharp"` |
| `outputDir` | `string` | `"models"` | Output directory where files will be created |
| `pkgName` | `string` | Language default | Package name (`"models"`) for Go, or Namespace (`"MigraDB.Models"`) for C# |

**Behavior & Features:**
- **Prefix Stripping**: Automatically strips `m_` (Master), `t_` (Transaction), `sys_` (System), `map_` (Mapping). E.g., `m_companies` becomes entity `Company`.
- **Singularization**: Automatically singularizes plural table names (e.g., `branches` -> `Branch`, `companies` -> `Company`).
- **Relationship Navigation**: Detects foreign key relationships in the diagram and generates typed pointer fields (e.g. `Company *Company` in `Branch struct`).
- **Nullable Handling**: Nullable database columns map to pointer types in Go (`*string`, `*time.Time`, `*uuid.UUID`).

**Example:**

```go
result, err := m.GenerateModels(
    "schema/drawdb.json",
    "go",
    "./internal/models",
    "models",
)
if err != nil {
    log.Fatalf("Model generation error: %v", err)
}

fmt.Printf("Generated %d files:\n", result.Count)
for _, f := range result.Files {
    fmt.Printf("  - %s\n", f)
}
```

---

## Constants

```go
const (
    VersionFormat        = "20060102150405" // YYYYMMDDHHmmss Go reference time format
    FileExtension        = ".sql"
    DefaultMigrationsDir = "./migrations"
)
```

## Error Handling Pattern

The Go library returns standard Go `error` values and populates result struct `Error` fields:

```go
result, err := m.Run()
if err != nil {
    log.Fatalf("Execution failed: %v", err)
}
if !result.Success {
    log.Fatalf("Migration SQL error: %s", result.Error)
}
```

## Thread Safety

The `Migrator` struct is **not** safe for concurrent calls on the same instance. Create separate instances per worker or synchronize method calls using a `sync.Mutex`.
