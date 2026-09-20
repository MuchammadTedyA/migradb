//go:build !windows && cgo

package migration

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/release -lmigration_engine
#include <stdlib.h>

extern void* migrator_new(const char* migrations_dir);
extern void* migrator_new_with_schema(const char* migrations_dir, const char* schema_dir);
extern void migrator_free(void* handle);
extern char* migrator_run(void* handle);
extern char* migrator_status(void* handle);
extern char* migrator_create(void* handle, const char* name, const char* content);
extern char* migrator_remove_pending(void* handle);
extern char* migrator_generate_models(void* handle, const char* schema_path, const char* target_lang, const char* output_dir, const char* package_name);
extern char* migrator_diff_drawdb(void* handle, const char* schema_path, const char* schema_dir, const char* migration_name, const char* dialect, int force_full);
extern char* migrator_plan_sync_drawdb(void* handle, const char* schema_path, const char* schema_dir, const char* dialect, int force_full);
extern void migrator_free_string(char* s);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"unsafe"
)

func NewMigrator(migrationsDir string) *Migrator {
	return NewMigratorWithSchema(migrationsDir, "./schema")
}

func NewMigratorWithSchema(migrationsDir, schemaDir string) *Migrator {
	if schemaDir == "" {
		schemaDir = "./schema"
	}
	cDir := C.CString(migrationsDir)
	defer C.free(unsafe.Pointer(cDir))
	cSchema := C.CString(schemaDir)
	defer C.free(unsafe.Pointer(cSchema))

	handle := C.migrator_new_with_schema(cDir, cSchema)
	if handle == nil {
		return nil
	}
	return &Migrator{
		handle:        handle,
		MigrationsDir: migrationsDir,
		SchemaDir:     schemaDir,
	}
}

func (m *Migrator) Close() {
	if m.handle != nil {
		C.migrator_free(m.handle)
		m.handle = nil
	}
}

func (m *Migrator) Run() (*RunResult, error) {
	cResult := C.migrator_run(m.handle)
	if cResult == nil {
		return nil, fmt.Errorf("failed to run migrations")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
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
	cResult := C.migrator_status(m.handle)
	if cResult == nil {
		return nil, fmt.Errorf("failed to get status")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
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
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	cContent := C.CString(content)
	defer C.free(unsafe.Pointer(cContent))

	cResult := C.migrator_create(m.handle, cName, cContent)
	if cResult == nil {
		return nil, fmt.Errorf("failed to create migration")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
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
	cResult := C.migrator_remove_pending(m.handle)
	if cResult == nil {
		return nil, fmt.Errorf("failed to remove pending migration")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
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
	cSchema := C.CString(schemaPath)
	defer C.free(unsafe.Pointer(cSchema))

	cLang := C.CString(targetLang)
	defer C.free(unsafe.Pointer(cLang))

	cOutDir := C.CString(outputDir)
	defer C.free(unsafe.Pointer(cOutDir))

	cPkg := C.CString(pkgName)
	defer C.free(unsafe.Pointer(cPkg))

	cResult := C.migrator_generate_models(m.handle, cSchema, cLang, cOutDir, cPkg)
	if cResult == nil {
		return nil, fmt.Errorf("failed to generate models")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
	var result GenerateModelsResult
	if err := json.Unmarshal([]byte(resultStr), &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}
	if !result.Success {
		return &result, fmt.Errorf(result.Error)
	}
	return &result, nil
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
	cSchema := C.CString(schemaPath)
	defer C.free(unsafe.Pointer(cSchema))

	cSchemaDir := C.CString(schemaDir)
	defer C.free(unsafe.Pointer(cSchemaDir))

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	cDialect := C.CString(dialect)
	defer C.free(unsafe.Pointer(cDialect))

	fullVal := C.int(0)
	if forceFull {
		fullVal = C.int(1)
	}

	cResult := C.migrator_diff_drawdb(m.handle, cSchema, cSchemaDir, cName, cDialect, fullVal)
	if cResult == nil {
		return nil, fmt.Errorf("failed to execute diff")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
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
	cSchema := C.CString(schemaPath)
	defer C.free(unsafe.Pointer(cSchema))

	cSchemaDir := C.CString(schemaDir)
	defer C.free(unsafe.Pointer(cSchemaDir))

	cDialect := C.CString(dialect)
	defer C.free(unsafe.Pointer(cDialect))

	fullVal := C.int(0)
	if forceFull {
		fullVal = C.int(1)
	}

	cResult := C.migrator_plan_sync_drawdb(m.handle, cSchema, cSchemaDir, cDialect, fullVal)
	if cResult == nil {
		return nil, fmt.Errorf("failed to plan sync")
	}
	defer C.migrator_free_string(cResult)

	resultStr := C.GoString(cResult)
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
