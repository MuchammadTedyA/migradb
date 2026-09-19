package migration

import (
	"os"
	"path/filepath"
	"testing"
)

func TestGenerateModels(t *testing.T) {
	m := NewMigrator("../migrations")
	if m == nil {
		t.Skip("Migrator unavailable on this environment (e.g. non-CGO build on darwin/linux), skipping test")
	}
	defer m.Close()

	schemaPath := filepath.Join("..", "schema", "drawdb.json")
	if _, err := os.Stat(schemaPath); os.IsNotExist(err) {
		t.Skip("schema/drawdb.json not found, skipping integration test")
	}

	tempDir, err := os.MkdirTemp("", "migradb-test-*")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tempDir)

	// 1. Test Go model generation
	goOut := filepath.Join(tempDir, "go")
	resGo, err := m.GenerateModels(schemaPath, "go", goOut, "models")
	if err != nil {
		t.Fatalf("GenerateModels(go) failed: %v", err)
	}
	if !resGo.Success || resGo.Count == 0 {
		t.Fatalf("Expected generated go files, got count %d", resGo.Count)
	}
	t.Logf("Generated %d Go model files", resGo.Count)

	// Check company.go exists and has content
	companyGo, err := os.ReadFile(filepath.Join(goOut, "company.go"))
	if err != nil {
		t.Fatalf("Failed to read company.go: %v", err)
	}
	if len(companyGo) == 0 {
		t.Fatal("company.go is empty")
	}

	// 2. Test TypeScript model generation
	tsOut := filepath.Join(tempDir, "ts")
	resTS, err := m.GenerateModels(schemaPath, "node", tsOut, "")
	if err != nil {
		t.Fatalf("GenerateModels(node) failed: %v", err)
	}
	if !resTS.Success || resTS.Count == 0 {
		t.Fatalf("Expected generated TS files, got count %d", resTS.Count)
	}
	t.Logf("Generated %d TS model files", resTS.Count)

	// 3. Test Python model generation
	pyOut := filepath.Join(tempDir, "py")
	resPy, err := m.GenerateModels(schemaPath, "python", pyOut, "")
	if err != nil {
		t.Fatalf("GenerateModels(python) failed: %v", err)
	}
	if !resPy.Success || resPy.Count == 0 {
		t.Fatalf("Expected generated Python files, got count %d", resPy.Count)
	}
	t.Logf("Generated %d Python model files", resPy.Count)

	// 4. Test C# model generation
	csOut := filepath.Join(tempDir, "csharp")
	resCS, err := m.GenerateModels(schemaPath, "csharp", csOut, "MigraDB.Models")
	if err != nil {
		t.Fatalf("GenerateModels(csharp) failed: %v", err)
	}
	if !resCS.Success || resCS.Count == 0 {
		t.Fatalf("Expected generated C# files, got count %d", resCS.Count)
	}
	t.Logf("Generated %d C# model files", resCS.Count)
}
