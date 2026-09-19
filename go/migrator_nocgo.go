//go:build !windows && !cgo

package migration

import (
	"fmt"
)

func NewMigrator(migrationsDir string) *Migrator {
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
