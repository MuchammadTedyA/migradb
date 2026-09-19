# Node.js Examples

Practical examples for using MigrDB in Node.js projects.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Express Integration](#express-integration)
- [Fastify Integration](#fastify-integration)
- [CLI Tool](#cli-tool)
- [Multi-Database](#multi-database)
- [Migration Generator](#migration-generator)

## Basic Usage

```javascript
const { Migrator } = require('migradb');

const m = new Migrator('./migrations');

// Check status
const status = m.status();
if (status.success) {
    console.log('Migration Status:');
    status.migrations.forEach(s => {
        const statusStr = s.applied ? 'APPLIED' : 'PENDING';
        console.log(`  [${statusStr}] ${s.version} - ${s.name}`);
    });
}

// Run migrations
const result = m.run();
if (result.success) {
    console.log(`\nApplied ${result.applied} migrations`);
}
```

## Express Integration

```javascript
const express = require('express');
const { Migrator } = require('migradb');

const app = express();
const m = new Migrator('./migrations');

// Run migrations on startup
const initResult = m.run();
if (initResult.success) {
    console.log(`Applied ${initResult.applied} migrations`);
} else {
    console.error('Migration failed:', initResult.error);
    process.exit(1);
}

// API endpoints
app.get('/api/migrations/status', (req, res) => {
    const status = m.status();
    res.json(status);
});

app.post('/api/migrations/run', (req, res) => {
    const result = m.run();
    res.json(result);
});

app.post('/api/migrations/create', (req, res) => {
    const { name, content } = req.body;
    const result = m.create(name, content);
    res.json(result);
});

app.listen(3000, () => {
    console.log('Server running on port 3000');
});
```

## Fastify Integration

```javascript
const fastify = require('fastify')({ logger: true });
const { Migrator } = require('migradb');

const m = new Migrator('./migrations');

// Run migrations on startup
fastify.addHook('onReady', async () => {
    const result = m.run();
    if (!result.success) {
        throw new Error(result.error);
    }
    fastify.log.info(`Applied ${result.applied} migrations`);
});

// Routes
fastify.get('/migrations/status', async () => {
    return m.status();
});

fastify.post('/migrations/run', async () => {
    return m.run();
});

fastify.post('/migrations/create', async (request) => {
    const { name, content } = request.body;
    return m.create(name, content);
});

// Start server
fastify.listen({ port: 3000 }, (err) => {
    if (err) {
        fastify.log.error(err);
        process.exit(1);
    }
});
```

## CLI Tool

```javascript
#!/usr/bin/env node

const { Migrator } = require('migradb');
const fs = require('fs');
const path = require('path');

function printUsage() {
    console.log('Usage: migrate <command> [options]');
    console.log('');
    console.log('Commands:');
    console.log('  run      Apply pending migrations');
    console.log('  status   Show migration status');
    console.log('  create   Create new migration file');
    console.log('  remove   Remove last pending migration');
    console.log('');
    console.log('Options:');
    console.log('  --dir    Migrations directory (default: ./migrations)');
}

function handleRun(m) {
    const result = m.run();

    if (!result.success) {
        console.error('Error:', result.error);
        process.exit(1);
    }

    if (result.applied === 0) {
        console.log('No pending migrations');
    } else {
        console.log(`Applied ${result.applied} migrations:`);
        result.migrations.forEach(m => {
            console.log(`  - ${m.version}: ${m.name}`);
        });
    }
}

function handleStatus(m) {
    const status = m.status();

    if (!status.success) {
        console.error('Error:', status.error);
        process.exit(1);
    }

    status.migrations.forEach(s => {
        const statusStr = s.applied ? 'APPLIED' : 'PENDING';
        console.log(`[${statusStr}] ${s.version} - ${s.name}`);
    });
}

function handleCreate(m, name) {
    if (!name) {
        console.error('Error: Migration name required');
        console.log('Usage: migrate create <name>');
        process.exit(1);
    }

    const result = m.create(name, '-- Add your SQL here\n');

    if (!result.success) {
        console.error('Error:', result.error);
        process.exit(1);
    }

    console.log(`Created: ${result.path}`);
}

function handleRemove(m) {
    const result = m.removePending();

    if (!result.success) {
        console.error('Error:', result.error);
        process.exit(1);
    }

    if (result.removed) {
        console.log(`Removed: ${result.removed}`);
    } else {
        console.log(result.message);
    }
}

function main() {
    const args = process.argv.slice(2);

    if (args.length === 0) {
        printUsage();
        return;
    }

    let migrationsDir = './migrations';
    const command = args[0];

    // Parse options
    for (let i = 1; i < args.length; i++) {
        if (args[i] === '--dir' && args[i + 1]) {
            migrationsDir = args[i + 1];
            i++;
        }
    }

    const m = new Migrator(migrationsDir);

    switch (command) {
        case 'run':
            handleRun(m);
            break;
        case 'status':
            handleStatus(m);
            break;
        case 'create':
            handleCreate(m, args.find(a => !a.startsWith('--')));
            break;
        case 'remove':
            handleRemove(m);
            break;
        default:
            console.error(`Unknown command: ${command}`);
            printUsage();
            process.exit(1);
    }
}

main();
```

## Multi-Database

```javascript
const { Migrator } = require('migradb');

const databases = {
    users: new Migrator('./migrations/users'),
    orders: new Migrator('./migrations/orders'),
    products: new Migrator('./migrations/products'),
};

async function runAllMigrations() {
    const results = {};

    for (const [name, m] of Object.entries(databases)) {
        const result = m.run();
        results[name] = result;

        if (result.success) {
            console.log(`${name}: Applied ${result.applied} migrations`);
        } else {
            console.error(`${name}: Error - ${result.error}`);
        }
    }

    return results;
}

async function checkAllStatus() {
    const status = {};

    for (const [name, m] of Object.entries(databases)) {
        status[name] = m.status();
    }

    return status;
}

runAllMigrations();
```

## Migration Generator

```javascript
const { Migrator } = require('migradb');

class TableMigration {
    constructor(migrator) {
        this.migrator = migrator;
    }

    createTable(tableName, columns) {
        const columnDefs = columns.map(col => {
            let def = `${col.name} ${col.type}`;

            if (!col.nullable) def += ' NOT NULL';
            if (col.default) def += ` DEFAULT ${col.default}`;
            if (col.primaryKey) def += ' PRIMARY KEY';
            if (col.unique) def += ' UNIQUE';

            return def;
        });

        const indexes = columns
            .filter(col => !col.primaryKey)
            .map(col => `CREATE INDEX IF NOT EXISTS idx_${tableName}_${col.name} ON ${tableName}(${col.name});`);

        const sql = `CREATE TABLE IF NOT EXISTS ${tableName} (\n    ${columnDefs.join(',\n    ')}\n);\n\n${indexes.join('\n')}`;

        return this.migrator.create(`create ${tableName} table`, sql);
    }

    addColumn(tableName, column) {
        let sql = `ALTER TABLE ${tableName} ADD COLUMN IF NOT EXISTS ${column.name} ${column.type}`;

        if (!column.nullable) sql += ' NOT NULL';
        if (column.default) sql += ` DEFAULT ${column.default}`;

        return this.migrator.create(`add ${column.name} to ${tableName}`, sql + ';');
    }

    addIndex(tableName, columns, unique = false) {
        const indexType = unique ? 'UNIQUE INDEX' : 'INDEX';
        const indexName = `idx_${tableName}_${columns.join('_')}`;
        const sql = `CREATE ${indexType} IF NOT EXISTS ${indexName} ON ${tableName}(${columns.join(', ')});`;

        return this.migrator.create(`add index to ${tableName}`, sql);
    }
}

// Usage
const m = new Migrator('./migrations');
const tm = new TableMigration(m);

// Create users table
tm.createTable('m_users', [
    { name: 'id', type: 'UUID', primaryKey: true, default: 'gen_random_uuid()' },
    { name: 'username', type: 'VARCHAR(100)', unique: true },
    { name: 'email', type: 'VARCHAR(255)', unique: true },
    { name: 'password_hash', type: 'TEXT' },
    { name: 'is_active', type: 'BOOLEAN', default: 'true' },
    { name: 'created_at', type: 'TIMESTAMPTZ', default: 'NOW()' },
]);

// Add column
tm.addColumn('m_users', {
    name: 'phone',
    type: 'VARCHAR(20)',
    nullable: true,
});

// Add index
tm.addIndex('m_users', ['email']);
tm.addIndex('m_users', ['email', 'username'], true);
```

## Environment-Based Configuration

```javascript
const { Migrator } = require('migradb');
const path = require('path');

const config = {
    migrationsDir: process.env.MIGRATIONS_DIR || path.join(__dirname, 'migrations'),
    environment: process.env.NODE_ENV || 'development',
};

const m = new Migrator(config.migrationsDir);

if (config.environment !== 'test') {
    const result = m.run();
    if (result.success) {
        console.log(`[${config.environment}] Applied ${result.applied} migrations`);
    } else {
        console.error(`[${config.environment}] Migration failed: ${result.error}`);
        process.exit(1);
    }
}

module.exports = m;
```
