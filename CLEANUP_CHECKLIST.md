# Phase 1 Cleanup Checklist

## Immediate Actions Required

### 1. Remove Unused Rust Code

**File**: `apps/api/src/models.rs`
- [ ] Remove `CreateUser` struct (lines 31-36) - Never used, user creation not implemented
- [ ] Remove `UserResponse` struct (lines 38-58) - Never returned by any endpoint
- [ ] Remove `Session` struct (line ~135) - Sessions handled differently now
- [ ] Remove `CreateSession` struct (line ~150) - Not used
- [ ] Remove `SessionStep` struct (line ~156) - Not used
- [ ] Remove `CompleteSession` struct (line ~163) - Not used
- [ ] Remove `AuditLog` struct (line ~172) - Audit logging not implemented
- [ ] Remove `CreateAuditLog` struct (line ~186) - Not used
- [ ] Remove `NavigationResponse` struct (line ~215) - Different response model used

**File**: `apps/api/src/middleware/auth.rs`
- [ ] Remove `require_admin_or_tech` function (line 67) - Only `require_admin` is used

**File**: `apps/api/src/routes/admin.rs`
- [ ] Remove `SessionsListQuery` struct (line 33) - Query not used

**File**: `apps/api/src/utils/jwt.rs`
- [ ] Remove `user_id` method from `Claims` impl (line 49) - Never called

### 2. Remove Outdated Files/Folders

- [ ] Delete `apps/api/web/` directory - Old ts-rs output, types now in `apps/web/src/types/`
- [ ] Archive or delete `refactor.md` - Historical doc, move to `/docs/archive/` if keeping
- [ ] Remove `.turbo/cache/` from git (add to .gitignore if not already)

### 3. Clean NPM Structure

**Issue**: Root package.json has dev dependencies that should be in app-specific package.json

**Root package.json** should only have:
```json
{
  "name": "equipment-troubleshooting",
  "version": "2.0.0",
  "private": true,
  "workspaces": [
    "apps/*"
  ],
  "scripts": {
    "dev": "turbo run dev",
    "build": "turbo run build",
    "test": "turbo run test"
  },
  "devDependencies": {
    "turbo": "latest"
  }
}
```

- [ ] Move all app-specific dependencies to `apps/web/package.json`
- [ ] Run `npm install` from root to clean up
- [ ] Delete root `node_modules` and reinstall

### 4. Update Dependencies

**Rust**:
- [ ] Update `sqlx` from 0.7.4 to 0.8.x (address deprecation warning)
- [ ] Run `cargo update` to get latest compatible versions
- [ ] Check for breaking changes in changelog

**NPM** (apps/web):
- [ ] Run `npm outdated` to see what's outdated
- [ ] Update non-breaking changes: `npm update`
- [ ] Check major version updates manually
- [ ] Test after each major update

### 5. Add .gitignore Entries

Add to `.gitignore` if not present:
```
# Build artifacts
target/
dist/
build/

# Dependencies
node_modules/

# Environment
.env
.env.local

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Turbo
.turbo/

# Generated types (if regenerating)
apps/api/web/

# Temporary
*.log
*.tmp
```

### 6. Clean Build Verification

After cleanup, verify:
```bash
# Backend clean build
cd apps/api
cargo clean
cargo build --release
# Should show 0 warnings

# Frontend clean build
cd ../web
rm -rf node_modules dist
npm install
npm run build
# Should show 0 warnings

# Root cleanup
cd ../..
rm -rf node_modules
npm install
```

## Expected Results

**Before Cleanup**:
- Rust: 12 warnings
- NPM: Many extraneous package warnings
- Outdated files present
- Messy file structure

**After Cleanup**:
- Rust: 0 warnings ✅
- NPM: 0 warnings ✅
- Clean file structure ✅
- All dependencies up to date ✅

## Files Safe to Remove (Confirmed)

These files/folders serve no purpose and can be deleted:
1. `apps/api/web/` - Outdated type generation output
2. `refactor.md` - Historical documentation
3. `.turbo/cache/` - Generated cache (recreated on build)

## Files to Keep

These are important for deployment/documentation:
1. `deploy/` - nginx and systemd configs
2. `DEPLOYMENT.md` - Deployment guide
3. `SSL_SETUP.md` - SSL setup guide
4. `.env.example` - Configuration template
5. `ENTERPRISE_ASSESSMENT.md` - This assessment
6. `README.md` - Project documentation

## Post-Cleanup Tasks

1. [ ] Update README.md with current project structure
2. [ ] Add CONTRIBUTING.md with development setup
3. [ ] Create TESTING.md for future test setup (Phase 2)
4. [ ] Document API endpoints (OpenAPI spec)
5. [ ] Set up GitHub Actions for CI/CD

## Notes

- Keep functionality unchanged - only remove unused/dead code
- Test application after each major change
- Commit changes incrementally
- Document any breaking changes

---

**Status**: Ready to execute
**Est. Time**: 2-3 hours
**Risk**: Low (only removing unused code)
