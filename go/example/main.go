package main

import (
	"fmt"
	"log"

	"github.com/centra/migration"
)

func main() {
	m := migration.NewMigrator("../migrations")
	defer m.Close()

	fmt.Println("=== Migration Status ===")
	status, err := m.Status()
	if err != nil {
		log.Fatal(err)
	}

	for _, s := range status.Migrations {
		statusStr := "PENDING"
		if s.Applied {
			statusStr = "APPLIED"
		}
		fmt.Printf("[%s] %s - %s\n", statusStr, s.Version, s.Name)
	}

	fmt.Println("\n=== Running Migrations ===")
	result, err := m.Run()
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("Applied %d migrations:\n", result.Applied)
	for _, m := range result.Migrations {
		fmt.Printf("  - %s: %s\n", m.Version, m.Name)
	}

	fmt.Println("\n=== Creating New Migration ===")
	createResult, err := m.Create("add products table", `CREATE TABLE IF NOT EXISTS m_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);`)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Created migration at: %s\n", createResult.Path)

	fmt.Println("\n=== Generating Model Classes ===")
	schemaPath := "../../centra-api/schema/drawdb.json"
	modelResult, err := m.GenerateModels(schemaPath, "go", "./generated_models", "models")
	if err != nil {
		fmt.Printf("Model generation skipped: %v\n", err)
	} else {
		fmt.Printf("Generated %d Go model classes in ./generated_models\n", modelResult.Count)
	}
}
