# Database Migrations

This directory contains SQLx migrations for the PostgreSQL database.

## Running Migrations

From the project root:

```bash
npm run migrate
```

Or directly with sqlx-cli:

```bash
sqlx migrate run --source apps/api/migrations
```

## Creating a New Migration

```bash
sqlx migrate add -r <migration_name> --source apps/api/migrations
```

This will create up and down migration files.

## Migration Files

Migrations will be created during Phase 2 of the migration plan.
See [MIGRATION.md](../../../MIGRATION.md) for details.
