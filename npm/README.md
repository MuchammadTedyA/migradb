# migradb

A fast, lightweight SQL database migration library with a Rust core engine for Node.js. Works seamlessly with PostgreSQL, MySQL, SQLite, or any custom SQL driver.

## Features

- ⚡ **Fast & Lightweight**: Native Rust parsing and validation engine via N-API.
- 🔄 **Universal Schema Sync (Approach 1)**: Sync live databases directly from DrawDB (`drawdb.json`) with zero migration regeneration on database engine switch.
- 🎯 **Multi-Dialect Support**: PostgreSQL by default, with complete native support for MySQL and SQLite.
- 📁 **Auto Directory Conventions**: Auto-creates `./schema` and `./migrations` if missing; allows custom paths.
- 🔄 **Universal Database Support**: Works with `pg` (PostgreSQL), `mysql2` (MySQL), `better-sqlite3` / `sqlite3`, or any query callback.
- 🛡️ **Transactional Safety**: Automatically executes each migration within its own atomic transaction.
- 📜 **Plain SQL Files**: Standard `.sql` files with timestamp-based ordering (`YYYYMMDDHHmmss_name.sql`).
- 📊 **Automatic History & Audit**: Manages `schema_migrations` tracking table and snapshots automatically.
- 🔷 **First-Class TypeScript Support**: Full type definitions included out of the box.

---

## 1. Installation

```bash
npm install migradb
```

Plus your favorite database driver:

```bash
# For PostgreSQL (default dialect)
npm install pg

# For MySQL
npm install mysql2

# For SQLite
npm install better-sqlite3
```

---

## 2. Universal Schema Sync (`syncDatabase`) - Recommended

Design your database schema visually in DrawDB, export the JSON, and let MigraDB synchronize your database on application start.

### Why Approach 1?
- **Zero Migration Regeneration**: If you change your database midway (e.g., SQLite in local testing -> PostgreSQL in production, or MySQL -> PostgreSQL), **you do not need to rewrite or regenerate any migrations**. Simply change your database connection, and MigraDB dynamically emits the correct DDL dialect!
- **Default Directory Conventions**: Automatically creates `./schema` and `./migrations` in the root project if they do not exist. If they already exist, files are placed directly inside.
- **Dialect Defaults**: Defaults to **PostgreSQL**. MySQL and SQLite are fully supported.
- **Audit Logging**: Saves timestamped audit migrations (`migrations/YYYYMMDDHHmmss_sync_drawdb.sql`) and snapshots (`schema/drawdb_snapshot.json`).

### A. With PostgreSQL (`pg`)
```javascript
const { Client } = require('pg');
const { syncDatabase } = require('migradb');

async function main() {
  const client = new Client({ connectionString: process.env.DATABASE_URL });
  await client.connect();

  // Synchronize against DrawDB schema
  const res = await syncDatabase(client, {
    schema: './schema/drawdb.json',    // Default schema path
    migrationsDir: './migrations',     // Auto-created if missing
    schemaDir: './schema',             // Auto-created if missing
    saveMigrationFile: true,           // Save audit .sql migration
    migrationName: 'init_schema',      // Migration slug
  });

  if (res.isEmpty) {
    console.log('✅ Schema is already up to date.');
  } else if (res.success) {
    console.log(`✅ Applied ${res.applied} statements! Audit file: ${res.migrationPath}`);
  } else {
    console.error('❌ Sync failed:', res.error);
  }

  await client.end();
}

main();
```

### B. With MySQL (`mysql2`)
```javascript
const mysql = require('mysql2/promise');
const { syncDatabase } = require('migradb');

async function main() {
  const connection = await mysql.createConnection({
    host: 'localhost', user: 'root', password: 'password', database: 'mydb',
  });

  const res = await syncDatabase(connection, {
    schema: './schema/drawdb.json',
    dialect: 'mysql', // Explicit or auto-detected
  });

  console.log(`Applied ${res.applied} statements to MySQL!`);
  await connection.end();
}

main();
```

### C. With SQLite (`better-sqlite3`)
```javascript
const Database = require('better-sqlite3');
const { syncDatabase } = require('migradb');

const db = new Database('app.db');
const res = await syncDatabase(db, {
  schema: './schema/drawdb.json',
  dialect: 'sqlite', // Auto-detected from better-sqlite3 instance
});
console.log(`Applied ${res.applied} statements to SQLite!`);
```

---

## 3. Applying Manual Migration Files (`runOnDatabase`)

Create a `migrations/` folder in your project and add your `.sql` files:

```text
my-project/
├── migrations/
│   ├── 20260901000000_create_users.sql
│   └── 20260901000100_add_roles.sql
├── package.json
└── index.js
```

**`migrations/20260901000000_create_users.sql`:**
```sql
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 3. Applying Migrations in Node.js

### A. With PostgreSQL (`pg`)

```javascript
const { Pool } = require('pg');
const { runOnDatabase, statusOnDatabase } = require('migradb');

const pool = new Pool({
  connectionString: process.env.DATABASE_URL || 'postgres://user:password@localhost:5432/mydb',
});

async function migrate() {
  // Check migration status (pending vs applied)
  const status = await statusOnDatabase(pool, './migrations');
  console.log('Status:', status.migrations);

  // Run all pending migrations in transactions
  const result = await runOnDatabase(pool, './migrations');
  if (result.success) {
    console.log(`Successfully applied ${result.applied} migration(s)!`);
  } else {
    console.error('Migration failed:', result.error);
    process.exit(1);
  }
}

migrate();
```

---

### B. With MySQL (`mysql2`)

```javascript
const mysql = require('mysql2/promise');
const { runOnDatabase } = require('migradb');

async function migrate() {
  const connection = await mysql.createConnection({
    host: 'localhost',
    user: 'root',
    password: 'password',
    database: 'mydb',
    multipleStatements: true, // Recommended for multi-statement migrations
  });

  const result = await runOnDatabase(connection, './migrations');
  console.log(`Applied ${result.applied} migrations`);
  await connection.end();
}

migrate();
```

---

### C. With SQLite (`better-sqlite3`)

```javascript
const Database = require('better-sqlite3');
const { runOnDatabase } = require('migradb');

const db = new Database('app.db');

async function migrate() {
  const result = await runOnDatabase(db, './migrations');
  console.log(`Applied ${result.applied} migrations`);
}

migrate();
```

---

## 4. Standalone Migration Script (`npm run migrate`)

A common practice is to create a dedicated script `migrate.js` in your project root:

**`migrate.js`:**
```javascript
const { Pool } = require('pg');
const { runOnDatabase } = require('migradb');

const pool = new Pool({ connectionString: process.env.DATABASE_URL });

runOnDatabase(pool, './migrations')
  .then((res) => {
    if (res.success) {
      console.log(`✅ Applied ${res.applied} migration(s).`);
      process.exit(0);
    } else {
      console.error(`❌ Migration failed: ${res.error}`);
      process.exit(1);
    }
  })
  .catch((err) => {
    console.error('Fatal error:', err);
    process.exit(1);
  });
```

Add this to your **`package.json`**:
```json
{
  "scripts": {
    "migrate": "node migrate.js",
    "start": "npm run migrate && node server.js"
  }
}
```

Now you can simply run:
```bash
npm run migrate
```

---

## 5. TypeScript Support

TypeScript definitions are included automatically:

```typescript
import { Pool } from 'pg';
import { runOnDatabase, statusOnDatabase, RunResult, StatusResult } from 'migradb';

const pool = new Pool({ connectionString: process.env.DATABASE_URL });

async function run(): Promise<void> {
  const status: StatusResult = await statusOnDatabase(pool, './migrations');
  for (const m of status.migrations) {
    console.log(`[${m.applied ? 'APPLIED' : 'PENDING'}] ${m.version} - ${m.name}`);
  }

  const result: RunResult = await runOnDatabase(pool, './migrations');
  if (result.success) {
    console.log(`Applied: ${result.applied}`);
  }
}

run();
```

---

## 6. Generate Migrations & Models from DrawDB (`drawdb.json`)

You can generate production-ready SQL migration files and TypeScript interfaces directly from your visual diagrams!

### A. In JavaScript / TypeScript Code:

```javascript
const { generateModels } = require('migradb');

// 1. Generate PostgreSQL migration SQL files directly from drawdb.json:
const sqlResult = generateModels('./schema/drawdb.json', 'sql', './migrations', 'postgres');
console.log('Generated SQL migrations:', sqlResult.files);

// 2. Generate TypeScript interface models from the same diagram:
const tsResult = generateModels('./schema/drawdb.json', 'ts', './src/models');
console.log('Generated TypeScript models:', tsResult.files);
```

### B. Using the npx CLI (Zero Setup):

If you prefer terminal commands without writing scripts, you can run MigraDB directly via `npx`:

```bash
# Generate PostgreSQL migrations into ./migrations:
npx migradb generate -s ./schema/drawdb.json -l sql -o ./migrations -p postgres

# Generate MySQL migrations:
npx migradb generate -s ./schema/drawdb.json -l sql -o ./migrations -p mysql

# Generate TypeScript interfaces:
npx migradb generate -s ./schema/drawdb.json -l ts -o ./src/models

# Create a new timestamped migration:
npx migradb create add_users_table --dir ./migrations

# Calculate schema diff against DrawDB diagram and generate incremental migration:
npx migradb diff --schema ./schema/drawdb.json --name add_user_profiles --dialect postgres
npx migradb diff --schema ./schema/drawdb.json --name add_user_profiles --dialect mysql
npx migradb diff --schema ./schema/drawdb.json --name add_user_profiles --dialect sqlite
```

---

## 7. How It Works Under the Hood

1. **Table Creation**: On first run, MigraDB creates a lightweight tracking table:
   ```sql
   CREATE TABLE IF NOT EXISTS schema_migrations (
       version VARCHAR(255) PRIMARY KEY,
       applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
   );
   ```
2. **Detection**: Reads all `.sql` files sorted by timestamp (`YYYYMMDDHHmmss`).
3. **Transaction Execution**: For each pending migration:
   - Starts a transaction (`BEGIN`).
   - Executes the SQL statements in the file.
   - Records the version in `schema_migrations`.
   - Commits the transaction (`COMMIT`).
   - If an error occurs, rolls back (`ROLLBACK`) so your database never gets stuck in a corrupt state.

---

## License

MIT © [MuchammadTedyA](https://github.com/MuchammadTedyA)
