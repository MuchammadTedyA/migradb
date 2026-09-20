#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { Migrator, generateModels } = require('../index');

function printHelp() {
  console.log(`
MigraDB CLI for Node.js (via npx)

USAGE:
    npx migradb <COMMAND> [OPTIONS]

COMMANDS:
    create <NAME> [--dir <PATH>]             Create a new timestamped migration file
    init [--dir <PATH>]                      Initialize a migrations folder with starter schema
    status [--dir <PATH>]                    Check migration files in the migrations directory
    generate -s <FILE> -l <LANG> -o <DIR>    Generate SQL or models from DrawDB JSON

EXAMPLES:
    npx migradb create add_users_table --dir ./migrations
    npx migradb generate -s drawdb.json -l sql -o ./migrations -p postgres
    npx migradb generate -s drawdb.json -l ts -o ./src/models
`);
}

function parseArgs(args) {
  const flags = {};
  const positional = [];

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '-s' || arg === '--schema') {
      flags.schema = args[++i];
    } else if (arg === '-l' || arg === '--lang' || arg === '--language') {
      flags.lang = args[++i];
    } else if (arg === '-o' || arg === '--out' || arg === '--output') {
      flags.out = args[++i];
    } else if (arg === '-p' || arg === '--pkg' || arg === '--package') {
      flags.pkg = args[++i];
    } else if (arg === '-d' || arg === '--dir' || arg === '--migrations-dir') {
      flags.dir = args[++i];
    } else if (arg === '-h' || arg === '--help') {
      flags.help = true;
    } else {
      positional.push(arg);
    }
  }

  return { flags, positional };
}

function main() {
  const args = process.argv.slice(2);
  if (args.length === 0) {
    printHelp();
    process.exit(0);
  }

  const command = args[0].toLowerCase();
  const { flags, positional } = parseArgs(args.slice(1));

  if (flags.help || command === 'help' || command === '--help') {
    printHelp();
    process.exit(0);
  }

  const dir = flags.dir || './migrations';

  switch (command) {
    case 'init': {
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      const initialFile = path.join(dir, '20260901000000_initial_schema.sql');
      if (!fs.existsSync(initialFile)) {
        fs.writeFileSync(
          initialFile,
          '-- Initial database schema\n-- Add your CREATE TABLE and schema definitions here\n'
        );
      }
      console.log(`✅ Initialized migrations directory at: ${dir}`);
      console.log(`   Created: ${initialFile}`);
      break;
    }

    case 'create': {
      const name = positional[0];
      if (!name) {
        console.error('Error: Migration name required. Example: npx migradb create add_users_table');
        process.exit(1);
      }
      const m = new Migrator(dir);
      const res = m.create(name, `-- Migration: ${name}\n-- Write your SQL statements below\n`);
      if (res.success) {
        console.log(`✅ Created migration: ${res.path}`);
      } else {
        console.error(`❌ Failed to create migration: ${res.error}`);
        process.exit(1);
      }
      break;
    }

    case 'status': {
      const m = new Migrator(dir);
      const res = m.status();
      if (!res.success) {
        console.error(`❌ Status check failed: ${res.error}`);
        process.exit(1);
      }
      console.log(`\nMigration Status (${dir}):`);
      console.log('-'.repeat(60));
      console.log('STATUS       VERSION            NAME');
      console.log('-'.repeat(60));
      if (res.migrations.length === 0) {
        console.log('No migration files found.');
      } else {
        res.migrations.forEach((item) => {
          const st = item.applied ? '[APPLIED]' : '[PENDING]';
          console.log(`${st.padEnd(12)} ${item.version.padEnd(18)} ${item.name}`);
        });
      }
      console.log('-'.repeat(60) + '\n');
      break;
    }

    case 'generate': {
      if (!flags.schema) {
        console.error('Error: --schema (-s) path is required.');
        process.exit(1);
      }
      const lang = flags.lang || 'sql';
      const outDir = flags.out || (lang === 'sql' ? dir : './src/models');
      const pkg = flags.pkg || (lang === 'sql' ? 'postgres' : undefined);

      const res = generateModels(flags.schema, lang, outDir, pkg);
      if (res.success) {
        console.log(`✅ Successfully generated ${res.files.length} file(s) in: ${outDir}`);
        res.files.forEach((f) => console.log(`   - ${f}`));
      } else {
        console.error(`❌ Codegen failed: ${res.error}`);
        process.exit(1);
      }
      break;
    }

    default:
      console.error(`Unknown command: ${command}`);
      printHelp();
      process.exit(1);
  }
}

main();

