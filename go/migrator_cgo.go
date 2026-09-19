//go:build !windows && cgo

package migration

/*
#cgo LDFLAGS: -L${SRCDIR}/../target/release -lmigration_engine
#include <stdlib.h>

extern void* migrator_new(const char* migrations_dir);
extern void migrator_free(void* handle);
extern char* migrator_run(void* handle);
extern char* migrator_status(void* handle);
extern char* migrator_create(void* handle, const char* name, const char* content);
extern char* migrator_remove_pending(void* handle);
extern char* migrator_generate_models(void* handle, const char* schema_path, const char* target_lang, const char* output_dir, const char* package_name);
extern void migrator_free_string(char* s);
*/
import "C"
import (
	"encoding/json"
	"fmt"
	"unsafe"
)

func NewMigrator(migrationsDir string) *Migrator {
	cDir := C.CString(migrationsDir)
	defer C.free(unsafe.Pointer(cDir))

	handle := C.migrator_new(cDir)
	if handle == nil {
		return nil
	}
	return &Migrator{handle: handle}
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
