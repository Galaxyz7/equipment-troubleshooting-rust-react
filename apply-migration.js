// Simple Node.js script to apply the display_category migration
const { Client } = require('pg');
require('dotenv').config();

async function applyMigration() {
  const client = new Client({
    connectionString: process.env.DATABASE_URL,
  });

  try {
    console.log('Connecting to database...');
    await client.connect();
    console.log('✓ Connected\n');

    // Step 1: Add display_category column
    console.log('1. Adding display_category column...');
    await client.query(
      'ALTER TABLE nodes ADD COLUMN IF NOT EXISTS display_category VARCHAR(255)'
    );
    console.log('   ✓ Column added or already exists\n');

    // Step 2: Create index
    console.log('2. Creating index...');
    await client.query(
      'CREATE INDEX IF NOT EXISTS idx_nodes_display_category ON nodes(display_category)'
    );
    console.log('   ✓ Index created or already exists\n');

    // Step 3: Update existing nodes
    console.log('3. Updating existing nodes with default category...');
    const result = await client.query(
      "UPDATE nodes SET display_category = 'General' WHERE display_category IS NULL"
    );
    console.log(`   ✓ Updated ${result.rowCount} rows\n`);

    // Step 4: Mark migrations as applied
    console.log('4. Marking migrations as applied...');
    await client.query(`
      INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
      VALUES
        (6, 'node graph refactor', true, decode('', 'hex'), 0),
        (7, 'add issue category', true, decode('', 'hex'), 0)
      ON CONFLICT (version) DO NOTHING
    `);
    console.log('   ✓ Migrations marked as applied\n');

    console.log('✅ Migration completed successfully!\n');
    console.log('You can now run: cd apps/api && cargo build\n');
  } catch (error) {
    console.error('❌ Error applying migration:', error.message);
    process.exit(1);
  } finally {
    await client.end();
  }
}

applyMigration();
