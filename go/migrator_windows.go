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
	procMigratorFree           *syscall.LazyProc
	procMigratorRun            *syscall.LazyProc
	procMigratorStatus         *syscall.LazyProc
	procMigratorCreate         *syscall.LazyProc
	procMigratorRemovePending  *syscall.LazyProc
	procMigratorFreeString     *syscall.LazyProc
	procMigratorGenerateModels *syscall.LazyProc
)

func init() {
	dllPath := findDLL()
	modMigrationEngine = syscall.NewLazyDLL(dllPath)

	procMigratorNew = modMigrationEngine.NewProc("migrator_new")
	procMigratorFree = modMigrationEngine.NewProc("migrator_free")
	procMigratorRun = modMigrationEngine.NewProc("migrator_run")
	procMigratorStatus = modMigrationEngine.NewProc("migrator_status")
	procMigratorCreate = modMigrationEngine.NewProc("migrator_create")
	procMigratorRemovePending = modMigrationEngine.NewProc("migrator_remove_pending")
	procMigratorFreeString = modMigrationEngine.NewProc("migrator_free_string")
	procMigratorGenerateModels = modMigrationEngine.NewProc("migrator_generate_models")
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
	cDir := cString(migrationsDir)
	handle, _, _ := procMigratorNew.Call(uintptr(unsafe.Pointer(cDir)))
	if handle == 0 {
		return nil
	}
	ptrHandle := *(*unsafe.Pointer)(unsafe.Pointer(&handle))
	return &Migrator{handle: ptrHandle}
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
		return &result, fmt.Errorf(result.Error)
	}
	return &result, nil
}
