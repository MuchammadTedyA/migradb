# Node.js API Reference

Complete API reference for the MigrDB Node.js module.

## Module: `migradb`

```javascript
const { Migrator, syncDatabase, detectDialect, runOnDatabase, statusOnDatabase, generateModels } = require('migradb');
// or
import { Migrator, syncDatabase, detectDialect, runOnDatabase, statusOnDatabase, generateModels } from 'migradb';
```

## Functions

### `syncDatabase(client, options?)`

Synchronizes a live database against a DrawDB schema file with dynamic multi-dialect translation and snapshot tracking.

```typescript
function syncDatabase(
    client: any,
    options?: {
        schema?: string;            // Path to drawdb.json (default: './schema/drawdb.json')
        migrationsDir?: string;     // Migrations directory (default: './migrations')
        schemaDir?: string;         // Schema snapshots directory (default: './schema')
        dialect?: string;           // 'postgres' (default), 'mysql', or 'sqlite'
        saveMigrationFile?: boolean;// Save audit .sql migration (default: true)
        migrationName?: string;     // Migration slug (default: 'sync_drawdb')
        forceFull?: boolean;        // Force baseline generation (default: false)
    }
): Promise<SyncResult>;
```

- **Features**:
  - Auto-creates `./schema` and `./migrations` folders if missing.
  - Generates dialect-accurate DDL for PostgreSQL (default), MySQL, and SQLite.
  - Zero migration regeneration required when switching database engines midway.
- **Returns**: `Promise<SyncResult>` containing `success`, `isEmpty`, `applied` statements count, `diffSummary`, `migrationPath`, `statements`, and optional `error`.

### `detectDialect(client)`

Infers the database dialect (`'postgres'`, `'mysql'`, or `'sqlite'`) from the client object.

```typescript
function detectDialect(client: any): string;
```

### `runOnDatabase(client, migrationsDir)`


Executes all pending migrations directly against a live database client within an atomic transaction.

```typescript
function runOnDatabase(client: any, migrationsDir: string): Promise<RunResult>;
```

- **Supported Clients**:
  - PostgreSQL: `pg.Client` or `pg.Pool`
  - MySQL: `mysql2/promise` Connection or Pool
  - SQLite: `better-sqlite3` Database instance
- **Returns**: `Promise<RunResult>` containing `success`, `applied` count, `migrations`, and optional `error`.

### `statusOnDatabase(client, migrationsDir)`

Checks migration status directly against a live database client.

```typescript
function statusOnDatabase(client: any, migrationsDir: string): Promise<StatusResult>;
```

- **Returns**: `Promise<StatusResult>` containing `success` and list of `MigrationStatus`.

### `generateModels(schemaPath, targetLang, outputDir, pkgOrNamespace?)`

Standalone function to generate strongly typed models from a DrawDB diagram export file.

```typescript
function generateModels(
    schemaPath: string,
    targetLang: string,
    outputDir: string,
    pkgOrNamespace?: string | null
): GenerateResult;
```

- **Parameters**:
  - `schemaPath`: Path to the DrawDB JSON export file
  - `targetLang`: Target language (`"node"`, `"ts"`, `"go"`, `"python"`, `"rust"`, `"csharp"`, `"sql"`)
  - `outputDir`: Destination directory for generated files
  - `pkgOrNamespace` (optional): Package name or namespace
- **Returns**: `GenerateResult` containing `success`, list of `files`, and optional `error`.

---

## Classes

### `Migrator`

The main class for managing database migration files and tracking.

```typescript
class Migrator {
    constructor(migrationsDir: string);
    run(): RunResult;
    status(): StatusResult;
    create(name: string, content: string): CreateResult;
    removePending(): RemoveResult;
    generateModels(schemaPath: string, targetLang: string, outputDir: string, pkgOrNamespace?: string | null): GenerateResult;
}
```

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| constructor | `new Migrator(migrationsDir: string)` | Creates instance with migrations directory |
| run | `run(): RunResult` | Applies all pending migrations against internal tracker |
| status | `status(): StatusResult` | Returns migration status |
| create | `create(name: string, content: string): CreateResult` | Creates a new migration file |
| removePending | `removePending(): RemoveResult` | Removes last pending migration |
| generateModels | `generateModels(schemaPath: string, targetLang: string, outputDir: string, pkgOrNamespace?: string \| null): GenerateResult` | Generates models from DrawDB JSON diagram |

---

## Interfaces

### `Migration`

Represents a single migration.

```typescript
interface Migration {
    version: string;
    name: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| version | `string` | 14-digit timestamp version (e.g., "20260901000000") |
| name | `string` | Migration slug (e.g., "initial_schema") |

---

### `MigrationStatus`

Represents the status of a migration.

```typescript
interface MigrationStatus {
    version: string;
    name: string;
    applied: boolean;
    appliedAt?: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| version | `string` | 14-digit timestamp version |
| name | `string` | Migration slug |
| applied | `boolean` | Whether the migration has been applied |
| appliedAt | `string | undefined` | ISO 8601 timestamp when applied |

---

### `RunResult`

Result from running migrations.

```typescript
interface RunResult {
    success: boolean;
    applied: number;
    migrations: Migration[];
    error?: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| success | `boolean` | Whether the operation succeeded |
| applied | `number` | Number of newly applied migrations |
| migrations | `Migration[]` | List of applied migrations |
| error | `string | undefined` | Error message if failed |

---

### `StatusResult`

Result from checking migration status.

```typescript
interface StatusResult {
    success: boolean;
    migrations: MigrationStatus[];
    error?: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| success | `boolean` | Whether the operation succeeded |
| migrations | `MigrationStatus[]` | List of all migrations with status |
| error | `string | undefined` | Error message if failed |

---

### `CreateResult`

Result from creating a migration.

```typescript
interface CreateResult {
    success: boolean;
    path: string;
    error?: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| success | `boolean` | Whether the operation succeeded |
| path | `string` | Path to the created migration file |
| error | `string | undefined` | Error message if failed |

---

### `RemoveResult`

Result from removing a pending migration.

```typescript
interface RemoveResult {
    success: boolean;
    removed?: string;
    message?: string;
    error?: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| success | `boolean` | Whether the operation succeeded |
| removed | `string | undefined` | Path of removed file |
| message | `string | undefined` | Informational message |
| error | `string | undefined` | Error message if failed |

---

### `GenerateResult`

Result from generating models from DrawDB schema.

```typescript
interface GenerateResult {
    success: boolean;
    files: string[];
    error?: string;
}
```

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| success | `boolean` | Whether code generation succeeded |
| files | `string[]` | List of relative file paths created |
| error | `string | undefined` | Error message if failed |

## Usage Examples

### Constructor

```javascript
const m = new Migrator('./migrations');
```

### Run Migrations

```javascript
const result = m.run();

if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
    result.migrations.forEach(m => {
        console.log(`  - ${m.version}: ${m.name}`);
    });
} else {
    console.error(result.error);
}
```

### Check Status

```javascript
const status = m.status();

if (status.success) {
    status.migrations.forEach(s => {
        const statusStr = s.applied ? 'APPLIED' : 'PENDING';
        console.log(`[${statusStr}] ${s.version} - ${s.name}`);
    });
}
```

### Create Migration

```javascript
const result = m.create('add products table', `
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL,
        price DECIMAL(10,2) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
`);

if (result.success) {
    console.log(`Created: ${result.path}`);
}
```

### Remove Pending Migration

```javascript
const result = m.removePending();

if (result.success) {
    if (result.removed) {
        console.log(`Removed: ${result.removed}`);
    } else {
        console.log(result.message);
    }
}
```

### Direct Database Migration (`runOnDatabase`)

Execute migrations against real database connections within a transaction:

```javascript
const { Client } = require('pg');
const { runOnDatabase, statusOnDatabase } = require('migradb');

async function migrate() {
    const client = new Client({ connectionString: 'postgresql://postgres:secret@localhost:5432/myapp' });
    await client.connect();

    const result = await runOnDatabase(client, './migrations');
    if (result.success) {
        console.log(`Applied ${result.applied} migrations!`);
    } else {
        console.error('Migration failed:', result.error);
    }

    const status = await statusOnDatabase(client, './migrations');
    console.log(`Total migrations: ${status.migrations.length}`);

    await client.end();
}

migrate();
```

### Model Generation (`generateModels`)

Generate TypeScript domain models and interfaces from DrawDB diagram export:

```javascript
const { generateModels } = require('migradb');

const result = generateModels('schema/drawdb.json', 'node', './src/models');
if (result.success) {
    console.log(`Generated ${result.files.length} model files:`, result.files);
} else {
    console.error('Generation failed:', result.error);
}
```

## Error Handling

All methods return result objects with a `success` boolean. Check this first before accessing other properties.

```javascript
const result = m.run();

if (!result.success) {
    // Handle error
    console.error(result.error);
    return;
}

// Handle success
console.log(`Applied ${result.applied} migrations`);
```

## TypeScript Support

The library includes native TypeScript declarations for the migration engine. All types are exported from `migradb`:

```typescript
import {
    Migrator,
    Migration,
    MigrationStatus,
    RunResult,
    StatusResult,
    CreateResult,
    RemoveResult,
    GenerateResult,
    runOnDatabase,
    statusOnDatabase,
    generateModels
} from 'migradb';
```

## Generated TypeScript Entity Models

When models are generated for Node.js / TypeScript (`targetLang: "node"`), MigrDB outputs strongly typed TypeScript definitions directly into your project:

- **Entity Files (`<entity>.ts`)**: Each database table generates an interface and optional class with proper primitive types (`string`, `number`, `boolean`, `Date`).
- **Foreign Key Navigation**: 1-to-many relationships are typed with navigation properties (e.g. `company?: Company;`).
- **Barrel Export (`index.ts`)**: Centralized re-export of all generated entity models for clean importing:
  ```typescript
  import { Company, Branch, Order } from './models';
  ```


## Platform Support

Pre-built binaries are available for:

| Platform | Architectures |
|----------|---------------|
| Linux | x64, arm64 (glibc & musl) |
| macOS | x64, arm64 |
| Windows | x64, ia32, arm64 |

## Performance

The Rust core provides:

- **Fast parsing**: Migration files parsed in microseconds
- **Low memory**: Minimal memory footprint
- **Concurrent-safe**: Thread-safe internal operations
