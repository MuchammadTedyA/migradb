# Node.js Examples

Practical, production-ready examples for using MigrDB in Node.js and TypeScript projects.

## Table of Contents

- [Real Database Drivers](#real-database-drivers)
  - [PostgreSQL (`pg`)](#postgresql-pg)
  - [MySQL (`mysql2`)](#mysql-mysql2)
  - [SQLite (`better-sqlite3`)](#sqlite-better-sqlite3)
- [Express Integration](#express-integration)
- [Fastify Integration](#fastify-integration)
- [Using the CLI (`npx migradb`)](#using-the-cli-npx-migradb)
- [DrawDB Model Generation](#drawdb-model-generation)
- [Multi-Database Migrations](#multi-database-migrations)
- [Dynamic Migration Creator](#dynamic-migration-creator)

---

## Real Database Drivers

MigrDB provides `runOnDatabase` and `statusOnDatabase` which directly accept live database clients, wrap migrations inside transactions, and maintain the `schema_migrations` audit table.

### PostgreSQL (`pg`)

```bash
npm install migradb pg
```

```javascript
const { Pool } = require('pg');
const { runOnDatabase, statusOnDatabase } = require('migradb');

const pool = new Pool({
    connectionString: process.env.DATABASE_URL || 'postgresql://postgres:secret@localhost:5432/myapp',
});

async function main() {
    // 1. Check current migration status
    const status = await statusOnDatabase(pool, './migrations');
    console.log('Migration Status:');
    status.migrations.forEach(m => {
        console.log(`  [${m.applied ? 'APPLIED' : 'PENDING'}] ${m.version} - ${m.name}`);
    });

    // 2. Apply all pending migrations in atomic transactions
    const result = await runOnDatabase(pool, './migrations');
    if (result.success) {
        console.log(`\nSuccessfully applied ${result.applied} migration(s)!`);
    } else {
        console.error('\nMigration failed:', result.error);
        process.exit(1);
    }

    await pool.end();
}

main().catch(console.error);
```

---

### MySQL (`mysql2`)

```bash
npm install migradb mysql2
```

```javascript
const mysql = require('mysql2/promise');
const { runOnDatabase, statusOnDatabase } = require('migradb');

async function main() {
    const connection = await mysql.createConnection({
        host: 'localhost',
        user: 'root',
        password: 'password',
        database: 'myapp',
    });

    const result = await runOnDatabase(connection, './migrations');
    if (result.success) {
        console.log(`Applied ${result.applied} migrations`);
    } else {
        console.error('Migration error:', result.error);
    }

    await connection.end();
}

main().catch(console.error);
```

---

### SQLite (`better-sqlite3`)

```bash
npm install migradb better-sqlite3
```

```javascript
const Database = require('better-sqlite3');
const { runOnDatabase, statusOnDatabase } = require('migradb');

const db = new Database('./app.db');

async function main() {
    const result = await runOnDatabase(db, './migrations');
    if (result.success) {
        console.log(`Applied ${result.applied} SQLite migrations`);
    } else {
        console.error('Error:', result.error);
    }

    db.close();
}

main().catch(console.error);
```

---

## Express Integration

Run pending migrations automatically when the Express server boots up:

```javascript
const express = require('express');
const { Pool } = require('pg');
const { runOnDatabase, statusOnDatabase, Migrator } = require('migradb');

const app = express();
app.use(express.json());

const pool = new Pool({
    connectionString: process.env.DATABASE_URL || 'postgresql://postgres:secret@localhost:5432/myapp',
});
const migrator = new Migrator('./migrations');

// API endpoint to inspect migration status
app.get('/api/migrations/status', async (req, res) => {
    try {
        const status = await statusOnDatabase(pool, './migrations');
        res.json(status);
    } catch (err) {
        res.status(500).json({ error: err.message });
    }
});

// API endpoint to trigger migrations manually
app.post('/api/migrations/run', async (req, res) => {
    try {
        const result = await runOnDatabase(pool, './migrations');
        res.json(result);
    } catch (err) {
        res.status(500).json({ error: err.message });
    }
});

// API endpoint to create a new migration file
app.post('/api/migrations/create', (req, res) => {
    const { name, content } = req.body;
    if (!name) {
        return res.status(400).json({ error: 'Migration name is required' });
    }
    const result = migrator.create(name, content || '-- Add SQL here\n');
    res.json(result);
});

// Bootstrap server with auto-migration
async function startServer() {
    console.log('Checking and running database migrations...');
    const result = await runOnDatabase(pool, './migrations');
    if (!result.success) {
        console.error('Failed to apply migrations:', result.error);
        process.exit(1);
    }
    console.log(`Applied ${result.applied} migrations.`);

    const PORT = process.env.PORT || 3000;
    app.listen(PORT, () => {
        console.log(`Server running on http://localhost:${PORT}`);
    });
}

startServer().catch(console.error);
```

---

## Fastify Integration

Use Fastify's `onReady` hook to execute migrations before accepting incoming traffic:

```javascript
const fastify = require('fastify')({ logger: true });
const { Pool } = require('pg');
const { runOnDatabase, statusOnDatabase } = require('migradb');

const pool = new Pool({
    connectionString: process.env.DATABASE_URL || 'postgresql://postgres:secret@localhost:5432/myapp',
});

// Run migrations during startup
fastify.addHook('onReady', async () => {
    fastify.log.info('Running database migrations...');
    const result = await runOnDatabase(pool, './migrations');
    if (!result.success) {
        fastify.log.error(`Migration failed: ${result.error}`);
        throw new Error(result.error);
    }
    fastify.log.info(`Applied ${result.applied} migrations.`);
});

// Clean up database connection on shutdown
fastify.addHook('onClose', async () => {
    await pool.end();
});

// Health & status routes
fastify.get('/migrations/status', async () => {
    return statusOnDatabase(pool, './migrations');
});

fastify.post('/migrations/run', async () => {
    return runOnDatabase(pool, './migrations');
});

const start = async () => {
    try {
        await fastify.listen({ port: 3000 });
    } catch (err) {
        fastify.log.error(err);
        process.exit(1);
    }
};

start();
```

---

## Using the CLI (`npx migradb`)

The `migradb` npm package bundles a CLI utility directly accessible via `npx`:

```bash
# 1. Initialize a new migrations project
npx migradb init --dir ./migrations

# 2. Create a timestamped migration
npx migradb create add_users_table --dir ./migrations

# 3. Check migration file status
npx migradb status --dir ./migrations

# 4. Generate TypeScript models from DrawDB visual schema
npx migradb generate --schema drawdb.json --lang node --out ./src/models
```

You can also add helper scripts to your `package.json`:

```json
{
  "scripts": {
    "migrate:new": "migradb create",
    "migrate:status": "migradb status",
    "codegen": "migradb generate --schema schema.json --lang node --out ./src/models"
  }
}
```

---

## DrawDB Model Generation

Export your diagram as JSON from [DrawDB](https://drawdb.app) and generate complete TypeScript definitions with relationships and exports:

```javascript
const { generateModels } = require('migradb');

// Generate TypeScript models into src/models
const result = generateModels(
    './schema/drawdb.json',  // Schema export file
    'node',                  // Target: "node" (TypeScript)
    './src/models'           // Target directory
);

if (result.success) {
    console.log(`Generated ${result.files.length} model files:`);
    result.files.forEach(file => console.log(`  - ${file}`));
} else {
    console.error('Model generation failed:', result.error);
}
```

Generated files include:
- Strongly typed TypeScript interfaces for each table
- Automatic table prefix stripping (`m_users` -> `User.ts`)
- Foreign key navigation properties
- Centralized barrel export in `index.ts`:

```typescript
// src/models/index.ts is generated automatically
import { User, Company, Order } from './models';
```

---

## Multi-Database Migrations

If your application uses multiple databases or multi-tenant schemas:

```javascript
const { Pool } = require('pg');
const { runOnDatabase } = require('migradb');

const tenants = [
    { name: 'tenant_a', url: process.env.TENANT_A_URL },
    { name: 'tenant_b', url: process.env.TENANT_B_URL },
];

async function migrateAllTenants() {
    for (const tenant of tenants) {
        console.log(`Migrating tenant: ${tenant.name}...`);
        const pool = new Pool({ connectionString: tenant.url });
        try {
            const result = await runOnDatabase(pool, './migrations');
            if (result.success) {
                console.log(`  [${tenant.name}] Applied ${result.applied} migrations`);
            } else {
                console.error(`  [${tenant.name}] Failed:`, result.error);
            }
        } finally {
            await pool.end();
        }
    }
}

migrateAllTenants().catch(console.error);
```

---

## Dynamic Migration Creator

Create formatted migration files programmatically:

```javascript
const { Migrator } = require('migradb');

const migrator = new Migrator('./migrations');

function createTableMigration(tableName, columns) {
    const columnDefs = columns.map(col => {
        let def = `    ${col.name} ${col.type}`;
        if (!col.nullable) def += ' NOT NULL';
        if (col.default) def += ` DEFAULT ${col.default}`;
        if (col.primaryKey) def += ' PRIMARY KEY';
        return def;
    });

    const sql = `CREATE TABLE IF NOT EXISTS ${tableName} (\n${columnDefs.join(',\n')}\n);\n`;
    return migrator.create(`create_${tableName}_table`, sql);
}

// Generate a migration file
const result = createTableMigration('products', [
    { name: 'id', type: 'SERIAL', primaryKey: true },
    { name: 'name', type: 'VARCHAR(255)', nullable: false },
    { name: 'price', type: 'DECIMAL(10,2)', nullable: false, default: '0.00' },
    { name: 'created_at', type: 'TIMESTAMP', default: 'CURRENT_TIMESTAMP' },
]);

console.log('Created migration:', result.path);
```
