# Node.js Guide

Complete guide for using MigrDB in Node.js projects.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Integration with Frameworks](#integration-with-frameworks)
- [TypeScript Support](#typescript-support)
- [Generated TypeScript Models](#generated-typescript-models)

## Installation

### NPM

```bash
npm install @centra/migradb
```

### Yarn

```bash
yarn add @centra/migradb
```

### PNPM

```bash
pnpm add @centra/migradb
```

## Quick Start

```javascript
const { Migrator } = require('@centra/migradb');

// Create migrator instance
const m = new Migrator('./migrations');

// Run all pending migrations
const result = m.run();

if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
} else {
    console.error(result.error);
}
```

### ES Modules

```javascript
import { Migrator } from '@centra/migradb';

const m = new Migrator('./migrations');
const result = m.run();
```

## API Reference

### Types

#### `Migrator`

The main class for managing migrations.

```typescript
class Migrator {
    constructor(migrationsDir: string);
    run(): RunResult;
    status(): StatusResult;
    create(name: string, content: string): CreateResult;
    removePending(): RemoveResult;
}
```

#### `Migration`

Represents a single migration.

```typescript
interface Migration {
    version: string;
    name: string;
}
```

#### `MigrationStatus`

Represents the status of a migration.

```typescript
interface MigrationStatus {
    version: string;
    name: string;
    applied: boolean;
    appliedAt?: string;
}
```

#### `RunResult`

Result from running migrations.

```typescript
interface RunResult {
    success: boolean;
    applied: number;
    migrations: Migration[];
    error?: string;
}
```

#### `StatusResult`

Result from checking migration status.

```typescript
interface StatusResult {
    success: boolean;
    migrations: MigrationStatus[];
    error?: string;
}
```

#### `CreateResult`

Result from creating a migration.

```typescript
interface CreateResult {
    success: boolean;
    path: string;
    error?: string;
}
```

#### `RemoveResult`

Result from removing a pending migration.

```typescript
interface RemoveResult {
    success: boolean;
    removed?: string;
    message?: string;
    error?: string;
}
```

### Methods

#### `constructor(migrationsDir: string)`

Creates a new Migrator instance.

**Parameters:**
- `migrationsDir` - Path to the migrations directory

**Example:**
```javascript
const m = new Migrator('./migrations');
```

#### `run(): RunResult`

Applies all pending migrations.

**Returns:**
- `RunResult` - Result of the operation

**Example:**
```javascript
const result = m.run();
if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
}
```

#### `status(): StatusResult`

Returns the status of all migrations.

**Returns:**
- `StatusResult` - Status of all migrations

**Example:**
```javascript
const status = m.status();
status.migrations.forEach(s => {
    console.log(`[${s.applied ? 'APPLIED' : 'PENDING'}] ${s.version} - ${s.name}`);
});
```

#### `create(name: string, content: string): CreateResult`

Creates a new migration file.

**Parameters:**
- `name` - Human-readable name for the migration
- `content` - SQL content of the migration

**Returns:**
- `CreateResult` - Result of the operation

**Example:**
```javascript
const result = m.create('add products table', `
    CREATE TABLE IF NOT EXISTS m_products (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(200) NOT NULL
    );
`);
```

#### `removePending(): RemoveResult`

Removes the last pending migration file.

**Returns:**
- `RemoveResult` - Result of the operation

**Example:**
```javascript
const result = m.removePending();
if (result.removed) {
    console.log(`Removed: ${result.removed}`);
}
```

## Examples

### Basic Migration Workflow

```javascript
const { Migrator } = require('@centra/migradb');

const m = new Migrator('./migrations');

// Check current status
const status = m.status();
if (status.success) {
    console.log('Current migration status:');
    status.migrations.forEach(s => {
        const statusStr = s.applied ? 'APPLIED' : 'PENDING';
        console.log(`  [${statusStr}] ${s.version} - ${s.name}`);
    });
}

// Run pending migrations
const result = m.run();
if (result.success) {
    console.log(`\nApplied ${result.applied} migrations:`);
    result.migrations.forEach(m => {
        console.log(`  - ${m.version}: ${m.name}`);
    });
}
```

### Creating a New Migration

```javascript
const result = m.create('add employees table', `
    CREATE TABLE IF NOT EXISTS m_employees (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES m_users(id),
        branch_id UUID REFERENCES m_branches(id),
        employee_code VARCHAR(20) NOT NULL UNIQUE,
        first_name VARCHAR(100) NOT NULL,
        last_name VARCHAR(100) NOT NULL,
        hire_date DATE NOT NULL,
        is_active BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_employees_user ON m_employees(user_id);
    CREATE INDEX IF NOT EXISTS idx_employees_branch ON m_employees(branch_id);
    CREATE INDEX IF NOT EXISTS idx_employees_code ON m_employees(employee_code);
`);

if (result.success) {
    console.log(`Created migration at: ${result.path}`);
}
```

### CLI Migration Tool

```javascript
#!/usr/bin/env node
const { Migrator } = require('@centra/migradb');
const path = require('path');

const migrationsDir = process.argv[2] || './migrations';
const command = process.argv[3] || 'status';

const m = new Migrator(migrationsDir);

switch (command) {
    case 'run':
        const runResult = m.run();
        if (runResult.success) {
            console.log(`Applied ${runResult.applied} migrations`);
        } else {
            console.error(runResult.error);
            process.exit(1);
        }
        break;

    case 'status':
        const statusResult = m.status();
        if (statusResult.success) {
            statusResult.migrations.forEach(s => {
                const statusStr = s.applied ? 'APPLIED' : 'PENDING';
                console.log(`[${statusStr}] ${s.version} - ${s.name}`);
            });
        }
        break;

    case 'create':
        const name = process.argv[4];
        if (!name) {
            console.error('Usage: migrate <dir> create <name>');
            process.exit(1);
        }
        const createResult = m.create(name, '-- Add your SQL here\n');
        if (createResult.success) {
            console.log(`Created: ${createResult.path}`);
        }
        break;

    default:
        console.error(`Unknown command: ${command}`);
        console.error('Commands: run, status, create');
        process.exit(1);
}
```

## Integration with Frameworks

### Express.js

```javascript
const express = require('express');
const { Migrator } = require('@centra/migradb');

const app = express();
const m = new Migrator('./migrations');

// Run migrations on startup
const result = m.run();
if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
} else {
    console.error('Migration failed:', result.error);
    process.exit(1);
}

// Migration status endpoint
app.get('/api/migrations/status', (req, res) => {
    const status = m.status();
    res.json(status);
});

// Run migrations endpoint
app.post('/api/migrations/run', (req, res) => {
    const result = m.run();
    res.json(result);
});

app.listen(3000, () => {
    console.log('Server running on port 3000');
});
```

### Fastify

```javascript
const fastify = require('fastify')({ logger: true });
const { Migrator } = require('@centra/migradb');

const m = new Migrator('./migrations');

// Run migrations on startup
fastify.addHook('onReady', async () => {
    const result = m.run();
    if (!result.success) {
        throw new Error(result.error);
    }
    fastify.log.info(`Applied ${result.applied} migrations`);
});

// Migration routes
fastify.get('/migrations/status', async () => {
    return m.status();
});

fastify.post('/migrations/run', async () => {
    return m.run();
});

fastify.listen({ port: 3000 }, (err) => {
    if (err) {
        fastify.log.error(err);
        process.exit(1);
    }
});
```

### NestJS

```typescript
import { Module, OnModuleInit } from '@nestjs/common';
import { Migrator } from '@centra/migradb';

@Injectable()
export class MigrationService implements OnModuleInit {
    private migrator: Migrator;

    constructor() {
        this.migrator = new Migrator('./migrations');
    }

    async onModuleInit() {
        const result = this.migrator.run();
        if (result.success) {
            console.log(`Applied ${result.applied} migrations`);
        } else {
            throw new Error(result.error);
        }
    }

    getStatus() {
        return this.migrator.status();
    }

    run() {
        return this.migrator.run();
    }
}

@Module({
    providers: [MigrationService],
    exports: [MigrationService],
})
export class MigrationModule {}
```

## TypeScript Support

The library includes TypeScript definitions out of the box.

```typescript
import { Migrator, RunResult, StatusResult } from '@centra/migradb';

const m = new Migrator('./migrations');

const result: RunResult = m.run();
if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
}

const status: StatusResult = m.status();
status.migrations.forEach(s => {
    console.log(`${s.version}: ${s.applied ? 'APPLIED' : 'PENDING'}`);
});
```

### Custom Type Definitions

```typescript
// types/migration.ts
export interface TableColumn {
    name: string;
    type: string;
    nullable?: boolean;
    default?: string;
    primaryKey?: boolean;
    unique?: boolean;
}

export interface MigrationOptions {
    timestamps?: boolean;
    ifNotExists?: boolean;
}

// utils/migration.ts
import { Migrator, CreateResult } from '@centra/migradb';

export function createTableMigration(
    migrator: Migrator,
    tableName: string,
    columns: TableColumn[]
): CreateResult {
    const columnDefs = columns.map(col => {
        let def = `${col.name} ${col.type}`;
        if (!col.nullable) def += ' NOT NULL';
        if (col.default) def += ` DEFAULT ${col.default}`;
        if (col.primaryKey) def += ' PRIMARY KEY';
        if (col.unique) def += ' UNIQUE';
        return def;
    });

    const sql = `CREATE TABLE IF NOT EXISTS ${tableName} (\n    ${columnDefs.join(',\n    ')}\n);`;

    return m.create(`create ${tableName} table`, sql);
}
```

## Generated TypeScript Models

You can generate strongly-typed TypeScript models directly from DrawDB visual schemas using the MigrDB core generator (with `--target node` or `targetLang: "node"`).

### Example Generated Entity (`branch.ts`)

```typescript
// Code generated by MigraDB. DO NOT EDIT.

export interface Branch {
    id: string;
    companyId: string;
    name: string;
    code: string;
    isActive: boolean;
    createdAt: Date;
    updatedAt: Date;

    // Navigation property (1-to-many relationship)
    company?: Company;
}
```

### Centralized Barrel Export (`index.ts`)

The generator provides a barrel file exporting all interfaces and types:

```typescript
export * from './company';
export * from './branch';
export * from './employee';
```

