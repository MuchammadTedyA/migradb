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

module.exports = {
  runOnDatabase,
  statusOnDatabase,
};
