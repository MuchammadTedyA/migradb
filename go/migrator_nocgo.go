//go:build !windows && !cgo

package migration

import (
	"fmt"
)

func NewMigrator(migrationsDir string) *Migrator {
	return nil
}

func NewMigratorWithSchema(migrationsDir, schemaDir string) *Migrator {
	return nil
}

func (m *Migrator) Close() {}

func (m *Migrator) Run() (*RunResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) Status() (*StatusResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) Create(name, content string) (*CreateResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) RemovePending() (*RemoveResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) GenerateModels(schemaPath, targetLang, outputDir, pkgName string) (*GenerateModelsResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) DiffDrawDB(schemaPath, name, dialect string, forceFull bool) (*DiffResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) DiffDrawDBWithSchemaDir(schemaPath, schemaDir, name, dialect string, forceFull bool) (*DiffResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) PlanSyncDrawDB(schemaPath, dialect string, forceFull bool) (*SyncPlan, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func (m *Migrator) PlanSyncDrawDBWithSchemaDir(schemaPath, schemaDir, dialect string, forceFull bool) (*SyncPlan, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func DiffDrawDB(schemaPath, migrationsDir, name, dialect string, forceFull bool) (*DiffResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}

func DiffDrawDBWithSchema(schemaPath, migrationsDir, schemaDir, name, dialect string, forceFull bool) (*DiffResult, error) {
	return nil, fmt.Errorf("cgo is required on non-windows platforms")
}
