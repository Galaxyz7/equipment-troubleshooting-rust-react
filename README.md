# Equipment Troubleshooting System

A modern web application for guided equipment troubleshooting using decision trees, built with Rust (Axum) + React + PostgreSQL.

> 📊 **Status:** Fully functional with admin panel and visual tree editor. [See refactor plan](refactor.md) for ongoing node-graph architecture improvements (30% complete).

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Database**: PostgreSQL (via Supabase)
- **Authentication**: JWT with admin role-based access
- **Tree Editor**: React Flow for visual decision tree editing

## Project Structure

```
equipment-troubleshooting-main/
├── apps/
│   ├── api/                 # Rust backend (Axum)
│   │   ├── src/
│   │   │   ├── routes/      # API route handlers
│   │   │   ├── models.rs    # Data models
│   │   │   ├── middleware/  # Auth & error handling
│   │   │   └── main.rs      # Server entry point
│   │   └── migrations/      # Database migrations
│   │
│   └── web/                 # React frontend
│       ├── src/
│       │   ├── pages/       # Route components
│       │   ├── components/  # Reusable components
│       │   ├── lib/         # API client & utilities
│       │   └── types/       # TypeScript types
│       └── public/
│
├── refactor.md             # Node-graph refactor plan (30% complete)
└── README.md               # This file
```

## Quick Start

### Prerequisites

- **Node.js** >= 18.0.0
- **Rust** >= 1.70.0
- **PostgreSQL** >= 14
- **npm** >= 9.0.0

### Installation

```bash
# Install dependencies
npm install

# Copy environment template
cp .env.example .env
# Edit .env with your database credentials

# Run database migrations
cd apps/api && cargo run --bin run_migration

# Start development servers (both API and Web)
npm run dev
```

The application will be available at:
- **Frontend:** http://localhost:5173
- **Backend API:** http://localhost:5000

## Development

### Available Commands

```bash
# Development
npm run dev              # Start both API and Web
npm run dev:web          # Start only React dev server
npm run dev:api          # Start only Rust API server

# Building
npm run build            # Build both for production
npm run build:web        # Build React app
npm run build:api        # Build Rust binary

# Testing
npm run test             # Run all tests
npm run test:web         # Run React tests
npm run test:api         # Run Rust tests

# Linting
npm run lint             # Lint all code
npm run lint:web         # Lint React code
npm run lint:api         # Lint Rust code (clippy)

# Formatting
npm run format           # Format all code
npm run format:web       # Format React code
npm run format:api       # Format Rust code

# Database
npm run migrate          # Run database migrations

# Cleanup
npm run clean            # Remove all build artifacts
```

## Features

### User-Facing
- **Interactive Troubleshooting**: Guided decision tree navigation for equipment issues
- **Category-Based**: Multiple equipment categories (Brush, Chemical, High Pressure, etc.)
- **Mobile-Friendly**: Responsive design with Tailwind CSS
- **Session Tracking**: History of troubleshooting paths

### Admin Panel
- **Issue Management**: View, create, edit, and delete equipment categories
- **Toggle Issues**: Enable/disable categories without deleting data
- **Visual Tree Editor**: React Flow-based editor for managing decision trees
- **Drag-and-Drop**: Rearrange nodes with saved custom layouts
- **Full CRUD**: Add/edit/delete questions, answers, and connections
- **Real-Time Editing**: Changes saved to database immediately

### Technical
- JWT-based authentication with admin roles
- Type-safe API with Rust + SQLx
- Modern React 18 with TypeScript
- PostgreSQL database via Supabase
- React Flow for interactive tree visualization
- Layout persistence via localStorage
- Auto-linking new issues to root question

## Database Schema

### Current Schema (Question/Answer Model)
- `users` - Admin authentication
- `questions` - Decision tree questions with categories
- `answers` - Answer options linking questions or providing conclusions

### Future Schema (Node-Graph Model) - 30% Migrated
- `nodes` - Unified questions + conclusions
- `connections` - Labeled edges between nodes
- See [refactor.md](refactor.md) for complete migration plan

## Admin Access

Default admin credentials:
- Email: `admin@gmail.com`
- Password: Set in `.env` as `ADMIN_PASSWORD_HASH`

Generate password hash:
```bash
cd apps/api
cargo run --bin hash_password "your-password"
```

## API Endpoints

### Public
- `POST /api/troubleshoot/start` - Start new session
- `POST /api/troubleshoot/:id/answer` - Submit answer choice
- `GET /api/troubleshoot/:id` - Get current session state

### Admin (Requires Auth)
- `POST /api/auth/login` - Admin login
- `GET /api/admin/issues` - List all categories
- `GET /api/admin/issues/:category/tree` - Get decision tree
- `POST /api/admin/issues` - Create new category
- `PATCH /api/admin/issues/:category/toggle` - Enable/disable category
- Full CRUD for questions, answers, nodes, connections

## Current Status

### Completed Features
- ✅ Full user troubleshooting flow
- ✅ Admin authentication with JWT
- ✅ Issues list with toggle/delete
- ✅ Visual tree editor with React Flow
- ✅ Drag-and-drop node positioning
- ✅ Layout persistence (localStorage)
- ✅ Full CRUD for decision trees

### In Progress
- 🔨 Node-graph architecture refactor (see [refactor.md](refactor.md))
  - 30% complete (database migration done)
  - Remaining: API routes, frontend updates
  - Simplifies admin UX (node-centric vs question/answer)

## Environment Variables

Required in `.env`:

```bash
DATABASE_URL=postgresql://user:pass@host:5432/database
JWT_SECRET=your-secret-key
ADMIN_USERNAME=admin@gmail.com
ADMIN_PASSWORD_HASH=argon2-hash
HOST=0.0.0.0
PORT=5000
FRONTEND_URL=http://localhost:5173
```

## Support

For ongoing development plans and architecture changes, see [refactor.md](refactor.md).
