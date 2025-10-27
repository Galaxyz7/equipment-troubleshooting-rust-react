# Contributing to Equipment Troubleshooting System

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Documentation](#documentation)

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, trolling, or discriminatory comments
- Publishing others' private information
- Unprofessional or unwelcome conduct

## Getting Started

### Prerequisites

**Required**:
- Node.js 18+ and npm 9+
- Rust 1.70+ and Cargo
- PostgreSQL 14+
- Git

**Recommended**:
- VS Code with Rust Analyzer and ESLint extensions
- Docker (for database setup)

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/equipment-troubleshooting-rust-react.git
   cd equipment-troubleshooting-rust-react
   ```

2. **Install dependencies**:
   ```bash
   # Install root dependencies
   npm install

   # Install API dependencies
   cd apps/api
   cargo build

   # Install web dependencies
   cd ../web
   npm install
   ```

3. **Set up environment variables**:
   ```bash
   # Copy example env files
   cp apps/api/.env.example apps/api/.env
   cp apps/web/.env.example apps/web/.env

   # Edit .env files with your local configuration
   ```

4. **Set up database**:
   ```bash
   # Create PostgreSQL database
   createdb equipment_troubleshooting

   # Run migrations
   cd apps/api
   sqlx migrate run

   # Seed initial data (optional)
   cargo run --bin seed
   ```

5. **Start development servers**:
   ```bash
   # Terminal 1: Start API server
   cd apps/api
   cargo run

   # Terminal 2: Start web dev server
   cd apps/web
   npm run dev
   ```

6. **Access the application**:
   - Web UI: http://localhost:5173
   - API: http://localhost:3001
   - API Docs: http://localhost:3001/swagger-ui

## Project Structure

```
equipment-troubleshooting-rust-react/
├── apps/
│   ├── api/                  # Rust/Axum backend
│   │   ├── src/
│   │   │   ├── routes/       # API route handlers
│   │   │   ├── models.rs     # Data models
│   │   │   ├── error.rs      # Error handling
│   │   │   ├── middleware/   # Auth, rate limiting, etc.
│   │   │   └── utils/        # Utilities (JWT, audit, cache)
│   │   ├── migrations/       # Database migrations
│   │   └── tests/            # Integration tests
│   └── web/                  # React/TypeScript frontend
│       ├── src/
│       │   ├── components/   # React components
│       │   ├── pages/        # Page components
│       │   ├── lib/          # Utilities (API client, logger)
│       │   ├── hooks/        # Custom React hooks
│       │   └── types/        # TypeScript type definitions
│       └── tests/            # Frontend tests
├── docs/                     # Additional documentation
└── README.md
```

## Development Workflow

### Branching Strategy

- `main`: Production-ready code
- `develop`: Integration branch for features
- `feature/feature-name`: New features
- `bugfix/bug-description`: Bug fixes
- `hotfix/critical-fix`: Urgent production fixes

### Workflow Steps

1. **Create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes**: Implement your feature/fix

3. **Test locally**: Ensure all tests pass

4. **Commit changes**: Follow commit guidelines

5. **Push to fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create Pull Request**: Submit PR for review

## Coding Standards

### Rust (Backend)

**Style Guide**:
- Follow Rust standard style guide
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Use meaningful variable and function names
- Document public APIs with doc comments

**Best Practices**:
```rust
/// Calculate the total cost including tax
///
/// # Arguments
/// * `subtotal` - The subtotal before tax
/// * `tax_rate` - Tax rate as a decimal (e.g., 0.08 for 8%)
///
/// # Example
/// ```
/// let total = calculate_total(100.0, 0.08);
/// assert_eq!(total, 108.0);
/// ```
pub fn calculate_total(subtotal: f64, tax_rate: f64) -> f64 {
    subtotal * (1.0 + tax_rate)
}
```

**Error Handling**:
- Use `ApiError` for all API errors
- Provide meaningful error messages
- Include context in error details
- Never panic in production code

### TypeScript/React (Frontend)

**Style Guide**:
- Use ESLint configuration (no-console rule enforced)
- Run `npm run lint` before committing
- Use TypeScript strict mode
- Prefer functional components with hooks

**Best Practices**:
```typescript
// Good: Functional component with typed props
interface UserCardProps {
  name: string;
  email: string;
  onDelete: (id: string) => void;
}

export function UserCard({ name, email, onDelete }: UserCardProps) {
  return (
    <div className="user-card">
      <h3>{name}</h3>
      <p>{email}</p>
      <button onClick={() => onDelete(id)}>Delete</button>
    </div>
  );
}
```

**State Management**:
- Use custom hooks for shared state (see `src/hooks/`)
- Prefer `useState` and `useContext` over external libraries
- Use `useCallback` for event handlers passed to children
- Use `useMemo` for expensive computations

**Logging**:
- **NEVER** use `console.log` or `console.error` directly
- Always use the logger utility:
  ```typescript
  import { logger } from '../lib/logger';

  logger.error('Failed to load data', { userId, error: getErrorMessage(err) });
  logger.info('Data loaded successfully', { count: data.length });
  ```

## Testing Requirements

### Backend Tests

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total() {
        assert_eq!(calculate_total(100.0, 0.08), 108.0);
    }
}
```

**Running Tests**:
```bash
cd apps/api

# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage
```

**Coverage Requirements**:
- Minimum 70% code coverage
- All public APIs must have tests
- Critical paths must be fully tested

### Frontend Tests

**Component Tests**:
```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { UserCard } from './UserCard';

describe('UserCard', () => {
  it('renders user information', () => {
    render(<UserCard name="John Doe" email="john@example.com" />);
    expect(screen.getByText('John Doe')).toBeInTheDocument();
  });
});
```

**Running Tests**:
```bash
cd apps/web

# Run tests
npm test

# Run with coverage
npm run test:coverage
```

**Coverage Requirements**:
- Minimum 70% code coverage
- All utility functions must have tests
- Critical user flows must be tested

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(api): add audit logging to admin endpoints

- Created audit.rs utility module
- Added audit logging to issue, node, connection endpoints
- Includes IP tracking and detailed context

Closes #123
```

```
fix(web): prevent console.log usage with ESLint rule

- Added no-console ESLint rule
- Replaced all console calls with logger utility
- Added structured logging with context

Fixes #456
```

## Pull Request Process

### Before Submitting

1. **Self-Review**:
   - Review your own code for obvious issues
   - Ensure code follows style guidelines
   - Check for console.log statements (frontend)
   - Remove debug code and comments

2. **Testing**:
   - All tests pass locally
   - New code has test coverage
   - Manual testing completed

3. **Documentation**:
   - Update relevant documentation
   - Add JSDoc/doc comments to new functions
   - Update CHANGELOG.md if applicable

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## How Has This Been Tested?
Describe testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No console.log statements
- [ ] All tests pass
```

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: At least one approval required
3. **Testing**: Reviewer tests changes locally
4. **Merge**: Squash and merge to main

## Documentation

### Code Documentation

**Rust**:
```rust
/// Authenticates a user and returns a JWT token
///
/// # Arguments
/// * `email` - User's email address
/// * `password` - User's password (will be hashed)
///
/// # Returns
/// * `Ok(String)` - JWT token on success
/// * `Err(ApiError)` - Authentication error
pub async fn authenticate(email: &str, password: &str) -> Result<String, ApiError> {
    // Implementation
}
```

**TypeScript**:
```typescript
/**
 * Validates an email address format
 * @param email - Email address to validate
 * @returns true if valid, false otherwise
 * @example
 * validateEmail('user@example.com') // true
 * validateEmail('invalid') // false
 */
export function validateEmail(email: string): boolean {
  // Implementation
}
```

### Documentation Files

- `README.md`: Project overview and quick start
- `ARCHITECTURE.md`: System architecture details
- `API.md`: API documentation
- `TESTING.md`: Testing guidelines
- `SECURITY.md`: Security policies
- `CONTRIBUTING.md`: This file

## Getting Help

### Resources

- **Documentation**: Check `/docs` folder
- **API Docs**: http://localhost:3001/swagger-ui
- **Issues**: GitHub Issues for questions

### Contact

- Open an issue for bugs or features
- Reach out to maintainers for complex questions
- Join discussions for architecture decisions

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to Equipment Troubleshooting System!
