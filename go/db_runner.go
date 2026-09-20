package migration

import (
	"context"
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"time"
)

var versionRegex = regexp.MustCompile(`^(\d+)(?:_(.+))?\.sql$`)

// RunDB applies all pending SQL migrations found in migrationsDir against any standard database/sql *sql.DB connection.
// Each migration runs within its own transaction. The schema_migrations tracking table is automatically created.
func RunDB(ctx context.Context, db *sql.DB, migrationsDir string) (*RunResult, error) {
	if err := initSchemaTable(ctx, db); err != nil {
		return &RunResult{Success: false, Error: err.Error()}, err
	}

	appliedVersions, err := getAppliedVersions(ctx, db)
	if err != nil {
		return &RunResult{Success: false, Error: err.Error()}, err
	}

	files, err := os.ReadDir(migrationsDir)
	if err != nil {
		return &RunResult{Success: false, Error: err.Error()}, err
	}

	type migrationEntry struct {
		version  string
		name     string
		filename string
	}

	var entries []migrationEntry
	for _, f := range files {
		if f.IsDir() || !strings.HasSuffix(f.Name(), ".sql") {
			continue
		}

		matches := versionRegex.FindStringSubmatch(f.Name())
		if len(matches) >= 2 {
			version := matches[1]
			name := ""
			if len(matches) >= 3 && matches[2] != "" {
				name = matches[2]
			} else {
				name = strings.TrimSuffix(f.Name(), ".sql")
			}
			entries = append(entries, migrationEntry{
				version:  version,
				name:     name,
				filename: f.Name(),
			})
		}
	}

	sort.Slice(entries, func(i, j int) bool {
		return entries[i].version < entries[j].version
	})

	var applied []Migration
	for _, entry := range entries {
		if appliedVersions[entry.version] {
			continue
		}

		fullPath := filepath.Join(migrationsDir, entry.filename)
		contentBytes, err := os.ReadFile(fullPath)
		if err != nil {
			return &RunResult{
				Success:    false,
				Applied:    len(applied),
				Migrations: applied,
				Error:      fmt.Sprintf("failed to read migration file %s: %v", entry.filename, err),
			}, err
		}

		// Execute in transaction
		tx, err := db.BeginTx(ctx, nil)
		if err != nil {
			return &RunResult{
				Success:    false,
				Applied:    len(applied),
				Migrations: applied,
				Error:      fmt.Sprintf("failed to begin transaction for %s: %v", entry.version, err),
			}, err
		}

		sqlContent := string(contentBytes)
		if strings.TrimSpace(sqlContent) != "" {
			if _, err := tx.ExecContext(ctx, sqlContent); err != nil {
				_ = tx.Rollback()
				return &RunResult{
					Success:    false,
					Applied:    len(applied),
					Migrations: applied,
					Error:      fmt.Sprintf("migration %s failed: %v", entry.version, err),
				}, err
			}
		}

		// Record migration
		recordSQL := fmt.Sprintf("INSERT INTO schema_migrations (version) VALUES ('%s');", entry.version)
		if _, err := tx.ExecContext(ctx, recordSQL); err != nil {
			_ = tx.Rollback()
			return &RunResult{
				Success:    false,
				Applied:    len(applied),
				Migrations: applied,
				Error:      fmt.Sprintf("failed to record version %s: %v", entry.version, err),
			}, err
		}

		if err := tx.Commit(); err != nil {
			return &RunResult{
				Success:    false,
				Applied:    len(applied),
				Migrations: applied,
				Error:      fmt.Sprintf("failed to commit migration %s: %v", entry.version, err),
			}, err
		}

		applied = append(applied, Migration{
			Version: entry.version,
			Name:    entry.name,
		})
	}

	return &RunResult{
		Success:    true,
		Applied:    len(applied),
		Migrations: applied,
	}, nil
}

// StatusDB returns the status of all migrations (applied and pending) against the database.
func StatusDB(ctx context.Context, db *sql.DB, migrationsDir string) (*StatusResult, error) {
	if err := initSchemaTable(ctx, db); err != nil {
		return &StatusResult{Success: false, Error: err.Error()}, err
	}

	appliedVersions, err := getAppliedVersions(ctx, db)
	if err != nil {
		return &StatusResult{Success: false, Error: err.Error()}, err
	}

	files, err := os.ReadDir(migrationsDir)
	if err != nil {
		return &StatusResult{Success: false, Error: err.Error()}, err
	}

	var statuses []MigrationStatus
	for _, f := range files {
		if f.IsDir() || !strings.HasSuffix(f.Name(), ".sql") {
			continue
		}

		matches := versionRegex.FindStringSubmatch(f.Name())
		if len(matches) >= 2 {
			version := matches[1]
			name := ""
			if len(matches) >= 3 && matches[2] != "" {
				name = matches[2]
			}
			statuses = append(statuses, MigrationStatus{
				Version: version,
				Name:    name,
				Applied: appliedVersions[version],
			})
		}
	}

	sort.Slice(statuses, func(i, j int) bool {
		return statuses[i].Version < statuses[j].Version
	})

	return &StatusResult{
		Success:    true,
		Migrations: statuses,
	}, nil
}

func initSchemaTable(ctx context.Context, db *sql.DB) error {
	createSQL := `CREATE TABLE IF NOT EXISTS schema_migrations (
		version VARCHAR(255) PRIMARY KEY,
		applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
	);`
	_, err := db.ExecContext(ctx, createSQL)
	return err
}

func getAppliedVersions(ctx context.Context, db *sql.DB) (map[string]bool, error) {
	rows, err := db.QueryContext(ctx, "SELECT version FROM schema_migrations;")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	applied := make(map[string]bool)
	for rows.Next() {
		var v string
		if err := rows.Scan(&v); err != nil {
			return nil, err
		}
		applied[v] = true
	}
	return applied, rows.Err()
}

// SyncDB synchronizes the database schema against a DrawDB schema file.
// It automatically detects the difference between the DrawDB schema and the existing snapshot,
// translates the diff into the configured database dialect (PostgreSQL by default, MySQL, or SQLite),
// executes the DDL transactionally, updates the schema_migrations tracking table, updates the snapshot,
// and optionally saves an audit migration file in migrationsDir.
func SyncDB(ctx context.Context, db *sql.DB, opts SyncOptions) (*SyncResult, error) {
	if opts.MigrationsDir == "" {
		opts.MigrationsDir = "./migrations"
	}
	if opts.SchemaPath == "" {
		opts.SchemaPath = filepath.Join("schema", "drawdb.json")
	}
	if opts.Dialect == "" {
		opts.Dialect = DialectPostgres
	}
	if opts.MigrationName == "" {
		opts.MigrationName = "sync_drawdb"
	}

	// Auto-create directories if missing
	schemaDir := filepath.Dir(opts.SchemaPath)
	if err := os.MkdirAll(schemaDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create schema directory %s: %w", schemaDir, err)
	}
	if err := os.MkdirAll(opts.MigrationsDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create migrations directory %s: %w", opts.MigrationsDir, err)
	}

	m := NewMigrator(opts.MigrationsDir)
	if m == nil {
		return nil, fmt.Errorf("failed to initialize migradb engine")
	}
	defer m.Close()

	plan, err := m.PlanSyncDrawDB(opts.SchemaPath, opts.Dialect, opts.ForceFull)
	if err != nil {
		return &SyncResult{Success: false, Error: err.Error()}, err
	}

	if plan.IsEmpty || len(plan.Statements) == 0 {
		return &SyncResult{
			Success:     true,
			IsEmpty:     true,
			Applied:     0,
			DiffSummary: plan.DiffSummary,
		}, nil
	}

	if err := initSchemaTable(ctx, db); err != nil {
		return &SyncResult{Success: false, Error: err.Error()}, err
	}

	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return &SyncResult{Success: false, Error: err.Error()}, fmt.Errorf("failed to begin transaction: %w", err)
	}

	for _, stmt := range plan.Statements {
		stmt = strings.TrimSpace(stmt)
		if stmt == "" {
			continue
		}
		if _, err := tx.ExecContext(ctx, stmt); err != nil {
			_ = tx.Rollback()
			return &SyncResult{
				Success:     false,
				DiffSummary: plan.DiffSummary,
				Error:       fmt.Sprintf("failed to execute statement '%s': %v", stmt, err),
			}, err
		}
	}

	// Record version in schema_migrations
	version := time.Now().UTC().Format("20060102150405")
	recordSQL := fmt.Sprintf("INSERT INTO schema_migrations (version) VALUES ('%s');", version)
	if _, err := tx.ExecContext(ctx, recordSQL); err != nil {
		_ = tx.Rollback()
		return &SyncResult{
			Success:     false,
			DiffSummary: plan.DiffSummary,
			Error:       fmt.Sprintf("failed to record migration version %s: %v", version, err),
		}, err
	}

	if err := tx.Commit(); err != nil {
		return &SyncResult{Success: false, Error: err.Error()}, fmt.Errorf("failed to commit transaction: %w", err)
	}

	var migPath string
	if opts.SaveMigrationFile {
		diffRes, err := m.DiffDrawDB(opts.SchemaPath, opts.MigrationName, opts.Dialect, opts.ForceFull)
		if err == nil && diffRes != nil {
			migPath = diffRes.MigrationPath
		}
	} else {
		// Update snapshot files directly
		schemaBytes, err := os.ReadFile(opts.SchemaPath)
		if err == nil {
			_ = os.WriteFile(filepath.Join(schemaDir, "drawdb_snapshot.json"), schemaBytes, 0644)
			_ = os.WriteFile(filepath.Join(opts.MigrationsDir, ".schema_snapshot.json"), schemaBytes, 0644)
		}
	}

	return &SyncResult{
		Success:       true,
		IsEmpty:       false,
		Applied:       len(plan.Statements),
		DiffSummary:   plan.DiffSummary,
		MigrationPath: migPath,
		Statements:    plan.Statements,
	}, nil
}

