#!/bin/bash
# Test Database Setup Script
# Creates and migrates the test database for integration tests

set -e

echo "🔧 Setting up test database..."

# Load test environment variables
export $(grep -v '^#' ../../.env.test | xargs)

# Extract database name from DATABASE_URL
DB_NAME=$(echo $DATABASE_URL | sed 's/.*\///')
DB_HOST=$(echo $DATABASE_URL | sed 's/.*@\(.*\):.*/\1/')
DB_PORT=$(echo $DATABASE_URL | sed 's/.*:\([0-9]*\)\/.*/\1/')
DB_USER=$(echo $DATABASE_URL | sed 's/.*\/\/\(.*\):.*/\1/')

echo "📦 Database: $DB_NAME"
echo "🌐 Host: $DB_HOST:$DB_PORT"
echo "👤 User: $DB_USER"

# Drop existing test database if it exists
echo "🗑️  Dropping existing test database (if exists)..."
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "DROP DATABASE IF EXISTS $DB_NAME;" || true

# Create test database
echo "📝 Creating test database..."
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "CREATE DATABASE $DB_NAME;"

# Run migrations
echo "🚀 Running migrations..."
cargo run --bin apply_migration

echo "✅ Test database setup complete!"
echo "💡 Run tests with: cargo test --all-features"
