from migradb import Migrator

m = Migrator("../migrations")

print("=== Migration Status ===")
status = m.status()

if status.success:
    for s in status.migrations:
        status_str = "APPLIED" if s.applied else "PENDING"
        print(f"[{status_str}] {s.version} - {s.name}")
else:
    print(status.error)

print("\n=== Running Migrations ===")
result = m.run()

if result.success:
    print(f"Applied {result.applied} migrations:")
    for mig in result.migrations:
        print(f"  - {mig.version}: {mig.name}")
else:
    print(result.error)

print("\n=== Creating New Migration ===")
create_result = m.create(
    "add products table",
    """CREATE TABLE IF NOT EXISTS m_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);""",
)

if create_result.success:
    print(f"Created migration at: {create_result.path}")
else:
    print(create_result.error)
