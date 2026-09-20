package migration

import "unsafe"

type Migration struct {
	Version string `json:"version"`
	Name    string `json:"name"`
}

type MigrationStatus struct {
	Version   string  `json:"version"`
	Name      string  `json:"name"`
	Applied   bool    `json:"applied"`
	AppliedAt *string `json:"applied_at,omitempty"`
}

type RunResult struct {
	Success    bool        `json:"success"`
	Applied    int         `json:"applied"`
	Migrations []Migration `json:"migrations"`
	Error      string      `json:"error,omitempty"`
}

type StatusResult struct {
	Success    bool              `json:"success"`
	Migrations []MigrationStatus `json:"migrations"`
	Error      string            `json:"error,omitempty"`
}

type CreateResult struct {
	Success bool   `json:"success"`
	Path    string `json:"path"`
	Error   string `json:"error,omitempty"`
}

type RemoveResult struct {
	Success bool   `json:"success"`
	Removed string `json:"removed,omitempty"`
	Message string `json:"message,omitempty"`
	Error   string `json:"error,omitempty"`
}

type GenerateModelsResult struct {
	Success bool     `json:"success"`
	Count   int      `json:"count"`
	Files   []string `json:"files"`
	Error   string   `json:"error,omitempty"`
}

const (
	DialectPostgres = "postgres"
	DialectMySQL    = "mysql"
	DialectSQLite   = "sqlite"
)

type DiffResult struct {
	Success       bool     `json:"success"`
	MigrationPath string   `json:"migration_path,omitempty"`
	DiffSummary   string   `json:"diff_summary"`
	IsEmpty       bool     `json:"is_empty"`
	Statements    []string `json:"statements,omitempty"`
	Error         string   `json:"error,omitempty"`
}

type SyncPlan struct {
	Success     bool     `json:"success"`
	IsEmpty     bool     `json:"is_empty"`
	DiffSummary string   `json:"diff_summary"`
	Statements  []string `json:"statements"`
	SQL         string   `json:"sql"`
	Dialect     string   `json:"dialect"`
	Error       string   `json:"error,omitempty"`
}

type SyncOptions struct {
	SchemaPath        string
	MigrationsDir     string
	Dialect           string // "postgres", "mysql", "sqlite"
	ForceFull         bool
	SaveMigrationFile bool
	MigrationName     string
}

type SyncResult struct {
	Success       bool     `json:"success"`
	IsEmpty       bool     `json:"is_empty"`
	Applied       int      `json:"applied"`
	DiffSummary   string   `json:"diff_summary"`
	MigrationPath string   `json:"migration_path,omitempty"`
	Statements    []string `json:"statements,omitempty"`
	Error         string   `json:"error,omitempty"`
}

type Migrator struct {
	handle unsafe.Pointer
}
