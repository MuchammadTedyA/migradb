const fs = require('fs');
const path = require('path');

const VERSION_REGEX = /^(\d+)(?:_(.+))?\.sql$/;

/**
 * Executes an SQL query against different types of database clients (pg, mysql2, sqlite, or callback).
 */
async function executeQuery(client, sqlQuery) {
  if (typeof client === 'function') {
    return await client(sqlQuery);
  }
  // pg / mysql2 style
  if (typeof client.query === 'function') {
    return await client.query(sqlQuery);
  }
  // better-sqlite3 style
  if (typeof client.exec === 'function') {
    return client.exec(sqlQuery);
  }
  // sqlite3 callback style
  if (typeof client.run === 'function') {
    return new Promise((resolve, reject) => {
      client.run(sqlQuery, (err) => {
        if (err) reject(err);
        else resolve();
      });
    });
  }
  throw new Error('Unsupported database client. Expected client with query(), exec(), or run(), or a query function.');
}

/**
 * Fetches applied versions from schema_migrations table.
 */
async function getAppliedVersions(client) {
  const query = 'SELECT version FROM schema_migrations;';
  if (typeof client === 'function') {
    const res = await client(query);
    return extractVersions(res);
  }
  if (typeof client.query === 'function') {
    const res = await client.query(query);
    // pg returns res.rows, mysql returns [rows, fields]
    const rows = res && res.rows ? res.rows : (Array.isArray(res) ? res[0] : res);
    return extractVersions(rows);
  }
  if (typeof client.prepare === 'function') {
    // better-sqlite3
    const rows = client.prepare(query).all();
    return extractVersions(rows);
  }
  return new Set();
}

function extractVersions(rows) {
  const set = new Set();
  if (Array.isArray(rows)) {
    for (const r of rows) {
      if (r && r.version) {
        set.add(String(r.version));
      }
    }
  }
  return set;
}

/**
 * Runs all pending migrations against the provided database connection.
 * @param {object|Function} client - Database connection (pg, mysql2, better-sqlite3, or query callback)
 * @param {string} migrationsDir - Path to directory containing .sql migration files
 */
async function runOnDatabase(client, migrationsDir) {
  try {
    // 1. Ensure schema_migrations table exists
    const initSql = `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version VARCHAR(255) PRIMARY KEY,
        applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
    `;
    await executeQuery(client, initSql);

    // 2. Fetch applied migrations
    const appliedSet = await getAppliedVersions(client);

    // 3. Scan migrations directory
    if (!fs.existsSync(migrationsDir)) {
      throw new Error(`Migrations directory not found: ${migrationsDir}`);
    }

    const files = fs.readdirSync(migrationsDir);
    const entries = [];

    for (const f of files) {
      if (!f.endsWith('.sql')) continue;
      const match = f.match(VERSION_REGEX);
      if (match) {
        entries.push({
          version: match[1],
          name: match[2] || f.replace(/\.sql$/, ''),
          filename: f,
        });
      }
    }

    entries.sort((a, b) => (a.version > b.version ? 1 : -1));

    const applied = [];
    for (const entry of entries) {
      if (appliedSet.has(entry.version)) {
        continue;
      }

      const filePath = path.join(migrationsDir, entry.filename);
      const sqlContent = fs.readFileSync(filePath, 'utf8');

      if (sqlContent.trim().length > 0) {
        await executeQuery(client, sqlContent);
      }

      // Record migration
      const recordSql = `INSERT INTO schema_migrations (version) VALUES ('${entry.version}');`;
      await executeQuery(client, recordSql);

      applied.push({
        version: entry.version,
        name: entry.name,
      });
    }

    return {
      success: true,
      applied: applied.length,
      migrations: applied,
    };
  } catch (err) {
    return {
      success: false,
      applied: 0,
      migrations: [],
      error: err.message || String(err),
    };
  }
}

/**
 * Inspects migration status against the database without applying changes.
 */
async function statusOnDatabase(client, migrationsDir) {
  try {
    const initSql = `
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version VARCHAR(255) PRIMARY KEY,
        applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
    `;
    await executeQuery(client, initSql);
    const appliedSet = await getAppliedVersions(client);

    if (!fs.existsSync(migrationsDir)) {
      throw new Error(`Migrations directory not found: ${migrationsDir}`);
    }

    const files = fs.readdirSync(migrationsDir);
    const list = [];

    for (const f of files) {
      if (!f.endsWith('.sql')) continue;
      const match = f.match(VERSION_REGEX);
      if (match) {
        list.push({
          version: match[1],
          name: match[2] || f.replace(/\.sql$/, ''),
          applied: appliedSet.has(match[1]),
        });
      }
    }

    list.sort((a, b) => (a.version > b.version ? 1 : -1));

    return {
      success: true,
      migrations: list,
    };
  } catch (err) {
    return {
      success: false,
      migrations: [],
      error: err.message || String(err),
    };
  }
}

function detectDialect(client) {
  if (!client) return 'postgres';
  if (typeof client.prepare === 'function' || (typeof client.exec === 'function' && !client.query)) {
    return 'sqlite';
  }
  if (client.threadId !== undefined || client.config?.database || (client.format && client.escape)) {
    return 'mysql';
  }
  return 'postgres';
}

/**
 * Synchronizes the database schema against a DrawDB schema file.
 * Dynamically compares schema against snapshot, translates DDL for target dialect,
 * executes in transaction, updates schema_migrations and snapshot, and optionally saves audit SQL.
 */
async function syncDatabase(client, options = {}) {
  const {
    schema = './schema/drawdb.json',
    migrationsDir = './migrations',
    schemaDir = './schema',
    dialect: specifiedDialect,
    saveMigrationFile = true,
    migrationName = 'sync_drawdb',
    forceFull = false,
  } = options;

  const dialect = specifiedDialect || detectDialect(client);

  if (!fs.existsSync(schemaDir)) {
    fs.mkdirSync(schemaDir, { recursive: true });
  }
  if (!fs.existsSync(migrationsDir)) {
    fs.mkdirSync(migrationsDir, { recursive: true });
  }

  const { planSyncDrawdb, diffDrawdb } = require('./index');

  const plan = planSyncDrawdb(schema, migrationsDir, schemaDir, dialect, forceFull);
  if (!plan.success && plan.error) {
    return {
      success: false,
      isEmpty: false,
      applied: 0,
      diffSummary: '',
      migrationPath: null,
      statements: [],
      error: plan.error,
    };
  }

  if (plan.isEmpty || !plan.statements || plan.statements.length === 0) {
    return {
      success: true,
      isEmpty: true,
      applied: 0,
      diffSummary: plan.diffSummary,
      statements: [],
      migrationPath: null,
    };
  }

  const initSql = `
    CREATE TABLE IF NOT EXISTS schema_migrations (
      version VARCHAR(255) PRIMARY KEY,
      applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
  `;
  await executeQuery(client, initSql);

  try {
    await executeQuery(client, 'BEGIN;');
    for (const stmt of plan.statements) {
      const trimmed = stmt.trim();
      if (trimmed) {
        await executeQuery(client, trimmed);
      }
    }

    const now = new Date();
    const pad = (n) => String(n).padStart(2, '0');
    const version = `${now.getUTCFullYear()}${pad(now.getUTCMonth() + 1)}${pad(now.getUTCDate())}${pad(now.getUTCHours())}${pad(now.getUTCMinutes())}${pad(now.getUTCSeconds())}`;
    const recordSql = `INSERT INTO schema_migrations (version) VALUES ('${version}');`;
    await executeQuery(client, recordSql);

    await executeQuery(client, 'COMMIT;');

    let migrationPath = null;
    if (saveMigrationFile) {
      const diffRes = diffDrawdb(schema, migrationsDir, schemaDir, migrationName, dialect, forceFull);
      if (diffRes && diffRes.migrationPath) {
        migrationPath = diffRes.migrationPath;
      }
    } else {
      const content = fs.readFileSync(schema, 'utf8');
      fs.writeFileSync(path.join(schemaDir, 'drawdb_snapshot.json'), content);
      fs.writeFileSync(path.join(migrationsDir, '.schema_snapshot.json'), content);
    }

    return {
      success: true,
      isEmpty: false,
      applied: plan.statements.length,
      diffSummary: plan.diffSummary,
      migrationPath,
      statements: plan.statements,
    };
  } catch (err) {
    try {
      await executeQuery(client, 'ROLLBACK;');
    } catch (_) {}
    return {
      success: false,
      isEmpty: false,
      applied: 0,
      diffSummary: plan.diffSummary,
      migrationPath: null,
      statements: plan.statements,
      error: err.message || String(err),
    };
  }
}

module.exports = {
  runOnDatabase,
  statusOnDatabase,
  syncDatabase,
  detectDialect,
};

