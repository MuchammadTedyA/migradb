const { Migrator } = require('@centra/migradb');

const m = new Migrator('../migrations');

console.log('=== Migration Status ===');
const status = m.status();

if (status.success) {
    for (const s of status.migrations) {
        const statusStr = s.applied ? 'APPLIED' : 'PENDING';
        console.log(`[${statusStr}] ${s.version} - ${s.name}`);
    }
} else {
    console.error(status.error);
}

console.log('\n=== Running Migrations ===');
const result = m.run();

if (result.success) {
    console.log(`Applied ${result.applied} migrations:`);
    for (const m of result.migrations) {
        console.log(`  - ${m.version}: ${m.name}`);
    }
} else {
    console.error(result.error);
}

console.log('\n=== Creating New Migration ===');
const createResult = m.create('add products table', `CREATE TABLE IF NOT EXISTS m_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);`);

if (createResult.success) {
    console.log(`Created migration at: ${createResult.path}`);
} else {
    console.error(createResult.error);
}
