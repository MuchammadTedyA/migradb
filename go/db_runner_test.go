package migration

import (
	"context"
	"database/sql"
	"database/sql/driver"
	"io"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
)

// In-memory mock driver for testing RunDB and StatusDB without external C dependencies.
type memDriver struct {
	mu       sync.Mutex
	versions map[string]bool
	tables   map[string]bool
}

var testMemDriver = &memDriver{
	versions: make(map[string]bool),
	tables:   make(map[string]bool),
}

func init() {
	sql.Register("mock_migradb", testMemDriver)
}

func (d *memDriver) Open(name string) (driver.Conn, error) {
	return &memConn{d: d}, nil
}

type memConn struct {
	d *memDriver
}

func (c *memConn) Prepare(query string) (driver.Stmt, error) {
	return &memStmt{c: c, query: query}, nil
}

func (c *memConn) Close() error {
	return nil
}

func (c *memConn) Begin() (driver.Tx, error) {
	return &memTx{}, nil
}

type memTx struct{}

func (t *memTx) Commit() error   { return nil }
func (t *memTx) Rollback() error { return nil }

type memStmt struct {
	c     *memConn
	query string
}

func (s *memStmt) Close() error { return nil }
func (s *memStmt) NumInput() int { return -1 }

func (s *memStmt) Exec(args []driver.Value) (driver.Result, error) {
	s.c.d.mu.Lock()
	defer s.c.d.mu.Unlock()

	q := strings.TrimSpace(s.query)
	if strings.HasPrefix(q, "CREATE TABLE IF NOT EXISTS schema_migrations") {
		s.c.d.tables["schema_migrations"] = true
	} else if strings.HasPrefix(q, "INSERT INTO schema_migrations") {
		// Extract version from VALUES ('...')
		start := strings.Index(q, "VALUES ('")
		if start != -1 {
			sub := q[start+9:]
			end := strings.Index(sub, "'")
			if end != -1 {
				v := sub[:end]
				s.c.d.versions[v] = true
			}
		}
	}
	return driver.RowsAffected(1), nil
}

func (s *memStmt) Query(args []driver.Value) (driver.Rows, error) {
	s.c.d.mu.Lock()
	defer s.c.d.mu.Unlock()

	var list []string
	for v := range s.c.d.versions {
		list = append(list, v)
	}
	return &memRows{versions: list, index: 0}, nil
}

type memRows struct {
	versions []string
	index    int
}

func (r *memRows) Columns() []string {
	return []string{"version"}
}

func (r *memRows) Close() error {
	return nil
}

func (r *memRows) Next(dest []driver.Value) error {
	if r.index >= len(r.versions) {
		return io.EOF
	}
	dest[0] = r.versions[r.index]
	r.index++
	return nil
}

func TestRunDBAndStatusDB(t *testing.T) {
	db, err := sql.Open("mock_migradb", "test")
	if err != nil {
		t.Fatalf("Failed to open mock db: %v", err)
	}
	defer db.Close()

	tempDir, err := os.MkdirTemp("", "migradb-runner-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tempDir)

	// Create 2 test migrations
	m1 := filepath.Join(tempDir, "20260101000000_init.sql")
	m2 := filepath.Join(tempDir, "20260101000100_add_table.sql")
	if err := os.WriteFile(m1, []byte("CREATE TABLE test_users (id INT);"), 0644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(m2, []byte("CREATE TABLE test_roles (id INT);"), 0644); err != nil {
		t.Fatal(err)
	}

	ctx := context.Background()

	// 1. Test StatusDB before running
	statusRes, err := StatusDB(ctx, db, tempDir)
	if err != nil {
		t.Fatalf("StatusDB failed: %v", err)
	}
	if len(statusRes.Migrations) != 2 {
		t.Fatalf("Expected 2 migrations, got %d", len(statusRes.Migrations))
	}
	if statusRes.Migrations[0].Applied || statusRes.Migrations[1].Applied {
		t.Fatalf("Expected all migrations to be pending initially")
	}

	// 2. Test RunDB
	runRes, err := RunDB(ctx, db, tempDir)
	if err != nil {
		t.Fatalf("RunDB failed: %v", err)
	}
	if !runRes.Success || runRes.Applied != 2 {
		t.Fatalf("Expected 2 applied migrations, got %d (success=%v)", runRes.Applied, runRes.Success)
	}

	// 3. Test StatusDB after running
	statusRes2, err := StatusDB(ctx, db, tempDir)
	if err != nil {
		t.Fatalf("StatusDB failed: %v", err)
	}
	if !statusRes2.Migrations[0].Applied || !statusRes2.Migrations[1].Applied {
		t.Fatalf("Expected all migrations to be applied after RunDB")
	}

	// 4. Test running again (idempotency - 0 applied)
	runRes2, err := RunDB(ctx, db, tempDir)
	if err != nil {
		t.Fatalf("Second RunDB failed: %v", err)
	}
	if runRes2.Applied != 0 {
		t.Fatalf("Expected 0 applied on second run, got %d", runRes2.Applied)
	}
}

func TestSyncDB(t *testing.T) {
	ctx := context.Background()
	db, err := sql.Open("mock_migradb", "test_sync")
	if err != nil {
		t.Fatalf("Failed to open mock DB: %v", err)
	}
	defer db.Close()

	tempDir, err := os.MkdirTemp("", "migradb_sync_test_*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	schemaPath := filepath.Join("..", "schema", "drawdb.json")
	if _, err := os.Stat(schemaPath); err != nil {
		t.Skip("schema/drawdb.json not found, skipping sync test")
	}

	opts := SyncOptions{
		SchemaPath:        schemaPath,
		MigrationsDir:     filepath.Join(tempDir, "migrations"),
		Dialect:           DialectPostgres,
		SaveMigrationFile: true,
		MigrationName:     "init_sync",
	}

	// 1. First sync - creates baseline
	res, err := SyncDB(ctx, db, opts)
	if err != nil {
		t.Fatalf("SyncDB failed: %v", err)
	}
	if !res.Success || res.IsEmpty || res.Applied == 0 {
		t.Fatalf("Expected initial sync to apply statements, got success=%v, applied=%d, empty=%v", res.Success, res.Applied, res.IsEmpty)
	}

	// 2. Second sync - should be empty (no changes)
	res2, err := SyncDB(ctx, db, opts)
	if err != nil {
		t.Fatalf("Second SyncDB failed: %v", err)
	}
	if !res2.Success || !res2.IsEmpty || res2.Applied != 0 {
		t.Fatalf("Expected second sync to be empty, got success=%v, applied=%d, empty=%v", res2.Success, res2.Applied, res2.IsEmpty)
	}
}

