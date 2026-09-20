//go:build windows

package migration

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"syscall"
	"unsafe"
)

var (
	modMigrationEngine         *syscall.LazyDLL
	procMigratorNew            *syscall.LazyProc
	procMigratorNewWithSchema  *syscall.LazyProc
	procMigratorFree           *syscall.LazyProc
	procMigratorRun            *syscall.LazyProc
	procMigratorStatus         *syscall.LazyProc
	procMigratorCreate         *syscall.LazyProc
	procMigratorRemovePending  *syscall.LazyProc
	procMigratorFreeString     *syscall.LazyProc
	procMigratorGenerateModels *syscall.LazyProc
	procMigratorDiffDrawDB     *syscall.LazyProc
	procMigratorPlanSyncDrawDB *syscall.LazyProc
)

func init() {
	dllPath := findDLL()
	modMigrationEngine = syscall.NewLazyDLL(dllPath)

	procMigratorNew = modMigrationEngine.NewProc("migrator_new")
	procMigratorNewWithSchema = modMigrationEngine.NewProc("migrator_new_with_schema")
	procMigratorFree = modMigrationEngine.NewProc("migrator_free")
	procMigratorRun = modMigrationEngine.NewProc("migrator_run")
	procMigratorStatus = modMigrationEngine.NewProc("migrator_status")
	procMigratorCreate = modMigrationEngine.NewProc("migrator_create")
	procMigratorRemovePending = modMigrationEngine.NewProc("migrator_remove_pending")
	procMigratorFreeString = modMigrationEngine.NewProc("migrator_free_string")
	procMigratorGenerateModels = modMigrationEngine.NewProc("migrator_generate_models")
	procMigratorDiffDrawDB = modMigrationEngine.NewProc("migrator_diff_drawdb")
	procMigratorPlanSyncDrawDB = modMigrationEngine.NewProc("migrator_plan_sync_drawdb")
}

func findDLL() string {
	if env := os.Getenv("MIGRATION_ENGINE_DLL"); env != "" {
		return env
	}

	candidates := []string{
		"migration_engine.dll",
		filepath.Join("..", "target", "release", "migration_engine.dll"),
		filepath.Join("..", "target", "debug", "migration_engine.dll"),
		filepath.Join("..", "..", "migradb", "target", "release", "migration_engine.dll"),
	}

	for _, p := range candidates {
		if _, err := os.Stat(p); err == nil {
			abs, err := filepath.Abs(p)
			if err == nil {
				return abs
			}
			return p
		}
	}
	return "migration_engine.dll"
}

func cString(s string) *byte {
	b := append([]byte(s), 0)
	return &b[0]
}

func goString(ptr uintptr) string {
	if ptr == 0 {
		return ""
	}
	p := *(*unsafe.Pointer)(unsafe.Pointer(&ptr))
	var length int
	for *(*byte)(unsafe.Add(p, length)) != 0 {
		length++
	}
	return string(unsafe.Slice((*byte)(p), length))
}

func NewMigrator(migrationsDir string) *Migrator {
	return NewMigratorWithSchema(migrationsDir, "./schema")
}

func NewMigratorWithSchema(migrationsDir, schemaDir string) *Migrator {
	if schemaDir == "" {
		schemaDir = "./schema"
	}
	cDir := cString(migrationsDir)
	cSchema := cString(schemaDir)
	var handle uintptr
	if procMigratorNewWithSchema != nil && procMigratorNewWithSchema.Find() == nil {
		handle, _, _ = procMigratorNewWithSchema.Call(uintptr(unsafe.Pointer(cDir)), uintptr(unsafe.Pointer(cSchema)))
	} else {
		handle, _, _ = procMigratorNew.Call(uintptr(unsafe.Pointer(cDir)))
	}
	if handle == 0 {
		return nil
	}
	ptrHandle := *(*unsafe.Pointer)(unsafe.Pointer(&handle))
	return &Migrator{
		handle:        ptrHandle,
		MigrationsDir: migrationsDir,
		SchemaDir:     schemaDir,
	}
}

func (m *Migrator) Close() {
	if m.handle != nil {
		procMigratorFree.Call(uintptr(m.handle))
		m.handle = nil
	}
}

func (m *Migrator) Run() (*RunResult, error) {
	r1, _, _ := procMigratorRun.Call(uintptr(m.handle))
	if r1 == 0 {
		return nil, fmt.Errorf("failed to run migrations")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result RunResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}
	if !result.Success {
		return &result, fmt.Errorf(result.Error)
	}
	return &result, nil
}

func (m *Migrator) Status() (*StatusResult, error) {
	r1, _, _ := procMigratorStatus.Call(uintptr(m.handle))
	if r1 == 0 {
		return nil, fmt.Errorf("failed to get status")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result StatusResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}
	if !result.Success {
		return &result, fmt.Errorf(result.Error)
	}
	return &result, nil
}

func (m *Migrator) Create(name, content string) (*CreateResult, error) {
	cName := cString(name)
	cContent := cString(content)
	r1, _, _ := procMigratorCreate.Call(uintptr(m.handle), uintptr(unsafe.Pointer(cName)), uintptr(unsafe.Pointer(cContent)))
	if r1 == 0 {
		return nil, fmt.Errorf("failed to create migration")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result CreateResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}
	if !result.Success {
		return &result, fmt.Errorf(result.Error)
	}
	return &result, nil
}

func (m *Migrator) RemovePending() (*RemoveResult, error) {
	r1, _, _ := procMigratorRemovePending.Call(uintptr(m.handle))
	if r1 == 0 {
		return nil, fmt.Errorf("failed to remove pending migration")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result RemoveResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}
	if !result.Success {
		return &result, fmt.Errorf(result.Error)
	}
	return &result, nil
}

func (m *Migrator) GenerateModels(schemaPath, targetLang, outputDir, pkgName string) (*GenerateModelsResult, error) {
	if targetLang == "" {
		targetLang = "go"
	}
	if outputDir == "" {
		outputDir = "models"
	}
	cSchema := cString(schemaPath)
	cLang := cString(targetLang)
	cOutDir := cString(outputDir)
	cPkg := cString(pkgName)

	r1, _, _ := procMigratorGenerateModels.Call(
		uintptr(m.handle),
		uintptr(unsafe.Pointer(cSchema)),
		uintptr(unsafe.Pointer(cLang)),
		uintptr(unsafe.Pointer(cOutDir)),
		uintptr(unsafe.Pointer(cPkg)),
	)
	if r1 == 0 {
		return nil, fmt.Errorf("failed to generate models")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result GenerateModelsResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}
	if !result.Success {
		return &result, fmt.Errorf("%s", result.Error)
	}
	return &result, nil
}

func boolToUintptr(b bool) uintptr {
	if b {
		return 1
	}
	return 0
}

func (m *Migrator) DiffDrawDB(schemaPath, name, dialect string, forceFull bool) (*DiffResult, error) {
	return m.DiffDrawDBWithSchemaDir(schemaPath, m.SchemaDir, name, dialect, forceFull)
}

func (m *Migrator) DiffDrawDBWithSchemaDir(schemaPath, schemaDir, name, dialect string, forceFull bool) (*DiffResult, error) {
	if dialect == "" {
		dialect = "postgres"
	}
	if name == "" {
		name = "sync_drawdb"
	}
	if schemaDir == "" {
		if m.SchemaDir != "" {
			schemaDir = m.SchemaDir
		} else {
			schemaDir = "./schema"
		}
	}
	cSchema := cString(schemaPath)
	cSchemaDir := cString(schemaDir)
	cName := cString(name)
	cDialect := cString(dialect)

	r1, _, _ := procMigratorDiffDrawDB.Call(
		uintptr(m.handle),
		uintptr(unsafe.Pointer(cSchema)),
		uintptr(unsafe.Pointer(cSchemaDir)),
		uintptr(unsafe.Pointer(cName)),
		uintptr(unsafe.Pointer(cDialect)),
		boolToUintptr(forceFull),
	)
	if r1 == 0 {
		return nil, fmt.Errorf("failed to execute diff")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result DiffResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse diff result: %w", err)
	}
	if !result.Success && result.Error != "" {
		return &result, fmt.Errorf("%s", result.Error)
	}
	return &result, nil
}

func (m *Migrator) PlanSyncDrawDB(schemaPath, dialect string, forceFull bool) (*SyncPlan, error) {
	return m.PlanSyncDrawDBWithSchemaDir(schemaPath, m.SchemaDir, dialect, forceFull)
}

func (m *Migrator) PlanSyncDrawDBWithSchemaDir(schemaPath, schemaDir, dialect string, forceFull bool) (*SyncPlan, error) {
	if dialect == "" {
		dialect = "postgres"
	}
	if schemaDir == "" {
		if m.SchemaDir != "" {
			schemaDir = m.SchemaDir
		} else {
			schemaDir = "./schema"
		}
	}
	cSchema := cString(schemaPath)
	cSchemaDir := cString(schemaDir)
	cDialect := cString(dialect)

	r1, _, _ := procMigratorPlanSyncDrawDB.Call(
		uintptr(m.handle),
		uintptr(unsafe.Pointer(cSchema)),
		uintptr(unsafe.Pointer(cSchemaDir)),
		uintptr(unsafe.Pointer(cDialect)),
		boolToUintptr(forceFull),
	)
	if r1 == 0 {
		return nil, fmt.Errorf("failed to plan sync")
	}
	defer procMigratorFreeString.Call(r1)

	resultStr := goString(r1)
	var result SyncPlan
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse sync plan: %w", err)
	}
	if !result.Success && result.Error != "" {
		return &result, fmt.Errorf("%s", result.Error)
	}
	return &result, nil
}

func DiffDrawDB(schemaPath, migrationsDir, name, dialect string, forceFull bool) (*DiffResult, error) {
	return DiffDrawDBWithSchema(schemaPath, migrationsDir, "./schema", name, dialect, forceFull)
}

func DiffDrawDBWithSchema(schemaPath, migrationsDir, schemaDir, name, dialect string, forceFull bool) (*DiffResult, error) {
	m := NewMigratorWithSchema(migrationsDir, schemaDir)
	if m == nil {
		return nil, fmt.Errorf("failed to initialize migradb engine")
	}
	defer m.Close()
	return m.DiffDrawDBWithSchemaDir(schemaPath, schemaDir, name, dialect, forceFull)
}
