# Node.js API Reference

Complete API reference for the MigrDB Node.js module.

## Module: `@centra/migradb`

```javascript
const { Migrator } = require('@centra/migradb');
// or
import { Migrator } from '@centra/migradb';
```

## Classes

### `Migrator`

The main class for managing database migrations.

```typescript
class Migrator {
    constructor(migrationsDir: string);
    run(): RunResult;
    status(): StatusResult;
    create(name: string, content: string): CreateResult;
    removePending(): RemoveResult;
}
```

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| constructor | `new Migrator(migrationsDir: string)` | Creates instance with migrations directory |
| run | `run(): RunResult` | Applies all pending migrations |
| status | `status(): StatusResult` | Returns migration status |
| create | `create(name: string, content: string): CreateResult` | Creates a new migration file |
| removePending | `removePending(): RemoveResult` | Removes last pending migration |

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

The library includes native TypeScript declarations for the migration engine. All types are exported from `@centra/migradb`:

```typescript
import {
    Migrator,
    Migration,
    MigrationStatus,
    RunResult,
    StatusResult,
    CreateResult,
    RemoveResult
} from '@centra/migradb';
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
