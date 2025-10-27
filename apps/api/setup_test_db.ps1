# Test Database Setup Script (PowerShell for Windows)
# Creates and migrates the test database for integration tests

Write-Host "🔧 Setting up test database..." -ForegroundColor Cyan

# Load test environment variables from .env.test
$envFile = "..\..\. env.test"
if (Test-Path $envFile) {
    Get-Content $envFile | ForEach-Object {
        if ($_ -notmatch '^\s*#' -and $_ -match '=') {
            $parts = $_ -split '=', 2
            $key = $parts[0].Trim()
            $value = $parts[1].Trim()
            [Environment]::SetEnvironmentVariable($key, $value, "Process")
        }
    }
}

$DATABASE_URL = $env:DATABASE_URL

# Parse database connection details
if ($DATABASE_URL -match 'postgresql://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)') {
    $DB_USER = $matches[1]
    $DB_PASS = $matches[2]
    $DB_HOST = $matches[3]
    $DB_PORT = $matches[4]
    $DB_NAME = $matches[5]

    Write-Host "📦 Database: $DB_NAME" -ForegroundColor Yellow
    Write-Host "🌐 Host: ${DB_HOST}:${DB_PORT}" -ForegroundColor Yellow
    Write-Host "👤 User: $DB_USER" -ForegroundColor Yellow

    # Set PGPASSWORD environment variable for psql
    $env:PGPASSWORD = $DB_PASS

    # Drop existing test database if it exists
    Write-Host "🗑️  Dropping existing test database (if exists)..." -ForegroundColor Yellow
    & psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>$null

    # Create test database
    Write-Host "📝 Creating test database..." -ForegroundColor Yellow
    & psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "CREATE DATABASE $DB_NAME;"

    if ($LASTEXITCODE -eq 0) {
        # Run migrations
        Write-Host "🚀 Running migrations..." -ForegroundColor Yellow
        cargo run --bin apply_migration

        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Test database setup complete!" -ForegroundColor Green
            Write-Host "💡 Run tests with: cargo test --all-features" -ForegroundColor Cyan
        } else {
            Write-Host "❌ Migration failed!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "❌ Database creation failed!" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "❌ Invalid DATABASE_URL format in .env.test" -ForegroundColor Red
    exit 1
}
