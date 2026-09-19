# migradb

A high-performance SQL database migration library with a Rust core engine, providing Node.js bindings via napi-rs.

## Installation

```bash
npm install migradb
```

## Quick Start

```javascript
const { Migrator } = require('migradb');

const m = new Migrator('./migrations');

// Check status
const status = m.status();
if (status.success) {
    status.migrations.forEach(s => {
        console.log(`[${s.applied ? 'APPLIED' : 'PENDING'}] ${s.version} - ${s.name}`);
    });
}

// Run migrations
const result = m.run();
if (result.success) {
    console.log(`Applied ${result.applied} migrations`);
} else {
    console.error(`Migration error: ${result.error}`);
}
```

## TypeScript Usage

```typescript
import { Migrator, RunResult, StatusResult } from 'migradb';

const m = new Migrator('./migrations');
const result: RunResult = m.run();
```

## Features

- **Timestamp-based versioning**: `YYYYMMDDHHmmss_slug.sql` format
- **Transactional safety**: Each migration executes within an isolated transaction
- **Pre-built binaries**: Powered by Rust via napi-rs for cross-platform performance

## License

MIT
