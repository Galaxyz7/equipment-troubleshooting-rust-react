# Phase 2: Testing Foundation - Implementation Plan

**Goal**: Achieve 80% code coverage across backend and frontend

**Status**: Ready to begin (Phase 1 complete ✅)

---

## Prerequisites Completed ✅

1. **Clean Codebase**
   - 0 Rust warnings
   - 0 ESLint warnings
   - 0 build errors
   - All React Hooks optimized with useCallback

2. **Testing Dependencies Added**
   - `axum-test` - Integration testing for Axum
   - `serial_test` - Sequential test execution
   - `http-body-util` - HTTP testing utilities

---

## Backend Testing (Rust) - Target: 75-85% Coverage

### 1. Test Infrastructure Setup

**Create**: `apps/api/tests/common/mod.rs`
```rust
// Test database setup
// Test app state initialization
// Helper functions for creating test users, tokens
// Database cleanup utilities
```

### 2. Authentication Tests

**Create**: `apps/api/tests/auth_tests.rs`

Test Coverage:
- ✅ User registration
- ✅ Login with valid credentials
- ✅ Login with invalid credentials
- ✅ Token refresh
- ✅ Token expiration handling
- ✅ Password hashing validation

### 3. Issues API Tests

**Create**: `apps/api/tests/issues_tests.rs`

Test Coverage:
- ✅ List all issues
- ✅ Get issue by category
- ✅ Create new issue
- ✅ Update issue metadata
- ✅ Delete issue
- ✅ Get issue graph structure
- ✅ Authorization checks (admin-only operations)

### 4. Nodes & Connections Tests

**Create**: `apps/api/tests/nodes_tests.rs`

Test Coverage:
- ✅ Create question node
- ✅ Create conclusion node
- ✅ Update node text
- ✅ Update node position
- ✅ Delete node
- ✅ Create connection between nodes
- ✅ Update connection label
- ✅ Delete connection
- ✅ Prevent circular dependencies

### 5. Troubleshooting Session Tests

**Create**: `apps/api/tests/troubleshoot_tests.rs`

Test Coverage:
- ✅ Start new session
- ✅ Start session with specific category
- ✅ Submit answer and navigate
- ✅ Navigate back in history
- ✅ Reach conclusion
- ✅ Session persistence
- ✅ Multiple concurrent sessions

### 6. Unit Tests for Utils

**Add to**: `apps/api/src/utils/jwt.rs`, `src/error.rs`

Test Coverage:
- ✅ JWT token generation
- ✅ JWT token verification
- ✅ Token expiration
- ✅ Error response formatting
- ✅ Password hashing and verification

---

## Frontend Testing (React + TypeScript) - Target: 70-80% Coverage

### 1. Test Infrastructure Setup

**Update**: `apps/web/vite.config.ts`
```typescript
test: {
  globals: true,
  environment: 'jsdom',
  setupFiles: './src/test/setup.ts',
  coverage: {
    provider: 'v8',
    reporter: ['text', 'html', 'lcov'],
    exclude: ['src/types/**', 'src/test/**']
  }
}
```

**Create**: `apps/web/src/test/setup.ts`
```typescript
// Mock API setup
// Test utilities
// Global test configuration
```

### 2. Component Tests

**Create**: `apps/web/src/components/__tests__/`

Test Files:
- `IssueCard.test.tsx`
- `TreeEditorModal.test.tsx`
- `Navbar.test.tsx`

Test Coverage:
- ✅ Component rendering
- ✅ User interactions (clicks, inputs)
- ✅ State management
- ✅ API call mocking
- ✅ Error handling display

### 3. Page Tests

**Create**: `apps/web/src/pages/__tests__/`

Test Files:
- `LoginPage.test.tsx`
- `TroubleshootPage.test.tsx`
- `IssuesListPage.test.tsx`
- `AdminDashboardPage.test.tsx`

Test Coverage:
- ✅ Page rendering
- ✅ Navigation flows
- ✅ Form submissions
- ✅ Loading states
- ✅ Error states
- ✅ Authentication checks

### 4. API Client Tests

**Create**: `apps/web/src/lib/__tests__/api.test.ts`

Test Coverage:
- ✅ API URL detection
- ✅ Request formatting
- ✅ Response parsing
- ✅ Error handling
- ✅ Token management

---

## Implementation Steps

### Step 1: Backend Test Infrastructure (30 min)
```bash
cd apps/api
mkdir -p tests/common
# Create common/mod.rs with test helpers
# Create .env.test with test database config
```

### Step 2: Backend Integration Tests (2-3 hours)
```bash
# Create auth_tests.rs
# Create issues_tests.rs
# Create nodes_tests.rs
# Create troubleshoot_tests.rs
cargo test
```

### Step 3: Frontend Test Setup (30 min)
```bash
cd apps/web
npm install --save-dev @testing-library/react @testing-library/jest-dom @testing-library/user-event jsdom @vitest/ui
# Create test/setup.ts
# Update vite.config.ts
```

### Step 4: Frontend Component Tests (2-3 hours)
```bash
# Create __tests__ directories
# Write component tests
# Write page tests
npm test
```

### Step 5: Coverage Analysis (30 min)
```bash
# Backend
cd apps/api
cargo tarpaulin --out Html

# Frontend
cd apps/web
npm run test -- --coverage
```

---

## Success Criteria

- ✅ **Backend**: 75-85% code coverage
- ✅ **Frontend**: 70-80% code coverage
- ✅ **Overall**: ~80% combined coverage
- ✅ All tests passing in CI/CD
- ✅ No flaky tests
- ✅ Fast test execution (<2 min backend, <30s frontend)

---

## Next Phase Preview

**Phase 3: Code Quality** (After testing complete)
- Add comprehensive documentation
- Implement structured logging
- Add request validation
- Error handling improvements
- Performance monitoring

---

## Quick Start Commands

```bash
# Backend tests
cd apps/api
cargo test --all-features

# Frontend tests
cd apps/web
npm test

# Coverage reports
cargo tarpaulin --out Html  # Backend
npm run test -- --coverage  # Frontend
```

---

**Ready to implement?** Start with Step 1: Backend Test Infrastructure
