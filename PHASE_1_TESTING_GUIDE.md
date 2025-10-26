# Phase 1: Critical Fixes - Testing Guide

**Version:** 1.0
**Date:** October 25, 2025
**Target Score:** 85/100 (B)

---

## 📋 Pre-Testing Checklist

### ✅ Verify Code Quality
- [x] Backend builds successfully: `cargo build`
- [x] Backend tests pass: `cargo test` (75/75 passing)
- [x] Frontend lints: `npm run lint` (0 errors)
- [x] Frontend tests pass: `npm test` (94/94 passing)

### ✅ Start Services

**Backend (Terminal 1):**
```bash
cd apps/api
cargo run
```
Expected: Server starts on `http://localhost:3000`

**Frontend (Terminal 2):**
```bash
cd apps/web
npm run dev
```
Expected: Vite dev server starts on `http://localhost:5173`

---

## 🧪 Test Cases

### Test 1: Backend Stats Endpoint - sessions_by_category ✅

**Objective:** Verify stats endpoint returns category breakdown instead of empty array

**Steps:**
1. Ensure backend is running
2. Login to get auth token:
   ```bash
   curl -X POST http://localhost:3000/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"admin@example.com","password":"admin123"}'
   ```
   Copy the `access_token` from response

3. Test stats endpoint:
   ```bash
   curl -X GET http://localhost:3000/api/admin/stats \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"
   ```

**Expected Response:**
```json
{
  "total_sessions": 10,
  "completed_sessions": 8,
  "abandoned_sessions": 2,
  "active_sessions": 0,
  "avg_steps_to_completion": 3.5,
  "most_common_conclusions": [
    {"conclusion": "Replace motor brushes", "count": 5},
    {"conclusion": "Check power supply", "count": 3}
  ],
  "sessions_by_category": [
    {"category": "motor_issues", "count": 7},
    {"category": "brush_problems", "count": 3}
  ]
}
```

**✅ Pass Criteria:**
- `sessions_by_category` is NOT empty `[]`
- Contains actual category data with counts
- Categories match issue categories in your database

---

### Test 2: Backend Sessions Endpoint - Query Parameters ✅

**Objective:** Verify sessions endpoint supports filtering

**Test 2a: Default Pagination**
```bash
curl -X GET "http://localhost:3000/api/admin/sessions" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
**Expected:** Returns first 50 sessions, page 1

**Test 2b: Filter by Status (Completed)**
```bash
curl -X GET "http://localhost:3000/api/admin/sessions?status=completed" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
**Expected:** Only completed sessions (where `completed_at` is not null)

**Test 2c: Filter by Date Range**
```bash
curl -X GET "http://localhost:3000/api/admin/sessions?start_date=2025-01-01&end_date=2025-12-31" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
**Expected:** Only sessions within date range

**Test 2d: Search by Tech ID**
```bash
curl -X GET "http://localhost:3000/api/admin/sessions?search=TECH" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
**Expected:** Sessions where `tech_identifier` or `client_site` contains "TECH"

**Test 2e: Filter by Category**
```bash
curl -X GET "http://localhost:3000/api/admin/sessions?category=motor_issues" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
**Expected:** Sessions where first step category is "motor_issues"

**Test 2f: Custom Pagination**
```bash
curl -X GET "http://localhost:3000/api/admin/sessions?page=2&page_size=10" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
**Expected:** Page 2, 10 items per page

**✅ Pass Criteria:**
- All filters work correctly
- Can combine multiple filters
- Pagination works
- Response includes `total_count`, `page`, `page_size`

---

### Test 3: Frontend - Analytics Page ✅

**Objective:** Verify Analytics page displays correctly with real data

**Steps:**
1. Navigate to `http://localhost:5173`
2. Login with admin credentials:
   - Email: `admin@example.com`
   - Password: `admin123`
3. Click **"📊 Analytics"** button in header
4. Verify URL changes to `/admin/analytics`

**Visual Checks:**
- [ ] Page header shows "Analytics Dashboard"
- [ ] "← Back to Issues" link visible
- [ ] 4 stat cards display:
  - 📊 Total Sessions (with count)
  - ✅ Completed (with percentage)
  - ⚠️ Abandoned (with percentage)
  - 👣 Avg Steps (with decimal)
- [ ] "Most Common Conclusions" chart shows:
  - Ranked bars (1, 2, 3...)
  - Purple→Blue gradient bars
  - Conclusion text and count
- [ ] "Sessions by Category" chart shows:
  - Ranked bars
  - Green→Emerald gradient bars
  - Category names and counts
- [ ] If active sessions > 0, blue banner shows active count
- [ ] Loading spinner appears briefly on first load
- [ ] No console errors

**✅ Pass Criteria:**
- All data displays correctly
- Charts render with gradients
- Clicking "← Back to Issues" navigates to `/admin`
- Page is responsive (try resizing browser)

---

### Test 4: Frontend - CreateIssueModal ✅

**Objective:** Verify modal replaces browser prompts with professional UI

**Steps:**
1. From `/admin` page, click **"+ Create New Issue"** button
2. Verify modal opens (should NOT see browser `prompt()` dialog)

**Modal Visual Checks:**
- [ ] Modal has purple→blue gradient header
- [ ] Title: "Create New Issue"
- [ ] Subtitle: "Add a new troubleshooting category"
- [ ] 3 form fields visible simultaneously:
  - Issue Name (required, red asterisk)
  - Display Category (dropdown, optional)
  - First Question (textarea, required, red asterisk)
- [ ] Auto-focused on "Issue Name" field
- [ ] Two buttons: "Cancel" and "Create Issue"

**Validation Tests:**

**Test 4a: Empty Submission**
1. Click "Create Issue" without filling anything
2. **Expected:** Red error messages appear:
   - "Issue name is required"
   - "First question is required"
3. Submit button should be disabled

**Test 4b: Name Too Short**
1. Enter name: "AB"
2. Tab out of field
3. **Expected:** Error: "Name must be at least 3 characters"

**Test 4c: Question Too Short**
1. Enter question: "Is motor"
2. Tab out of field
3. **Expected:** Error: "Question must be at least 10 characters"

**Test 4d: Category ID Preview**
1. Enter name: "Brush Problems"
2. **Expected:** Shows: "Category ID: `brush_problems`"

**Test 4e: Successful Creation**
1. Fill form:
   - Name: "Test Issue XYZ"
   - Display Category: "Electrical"
   - First Question: "Is there power to the device?"
2. Click "Create Issue"
3. **Expected:**
   - Button shows "Creating..." with spinner
   - Modal closes on success
   - New issue appears in issues list
   - Issue is sorted alphabetically

**Test 4f: Cancel**
1. Fill form partially
2. Click "Cancel"
3. **Expected:**
   - Modal closes
   - Form resets
   - No issue created

**Test 4g: Backdrop Click**
1. Open modal
2. Click outside modal (on dark backdrop)
3. **Expected:** Modal stays open (prevents accidental close)

**✅ Pass Criteria:**
- NO browser `prompt()` dialogs appear
- All validation works
- Category ID auto-generates correctly
- Form resets after submit/cancel
- Loading states work

---

### Test 5: Frontend - Improved Error Messages ✅

**Objective:** Verify errors display in UI instead of `alert()` dialogs

**Test 5a: Delete Error**
1. From `/admin` page
2. Try to delete an issue that doesn't exist (manually call API with fake ID)
3. **Expected:**
   - Red error banner appears at top: "Failed to delete issue. Please try again."
   - NO `alert()` popup
   - Error includes helpful message from API

**Test 5b: Toggle Error**
1. Disconnect internet or stop backend
2. Try to toggle an issue status
3. **Expected:**
   - Red error banner: "Failed to toggle issue status. Please try again."
   - NO `alert()` popup

**Test 5c: Create Error (Duplicate)**
1. Create issue with name that already exists
2. **Expected:**
   - Error appears IN modal (not alert)
   - Shows API error message
   - Modal stays open
   - Form data preserved

**✅ Pass Criteria:**
- NO `alert()` popups for errors
- Errors display in red banners
- Error messages are helpful and specific
- Errors extract message from API response

---

### Test 6: Regression Testing ✅

**Objective:** Ensure existing functionality still works

**Test 6a: Issue Toggle**
1. Toggle issue active/inactive
2. **Expected:** Status changes, UI updates

**Test 6b: Issue Edit**
1. Click "Edit" on an issue
2. **Expected:** TreeEditorModal opens

**Test 6c: Issue Delete**
1. Click "Delete" on an issue
2. Confirm deletion
3. **Expected:** Issue removed from list

**Test 6d: Issue Test**
1. Click "Test" on an issue
2. **Expected:** Opens troubleshooting flow in new tab

**✅ Pass Criteria:**
- All existing features work as before
- No regressions introduced

---

## 🎯 Final Verification Checklist

### Backend ✅
- [ ] Stats endpoint returns `sessions_by_category` with data
- [ ] Sessions endpoint supports all query parameters
- [ ] All 75 backend tests pass
- [ ] No compilation warnings (except 1 expected unused field warning)

### Frontend ✅
- [ ] Analytics page displays correctly with charts
- [ ] CreateIssueModal replaces prompts
- [ ] Form validation works
- [ ] Error messages display in UI banners
- [ ] All 94 frontend tests pass
- [ ] No ESLint errors
- [ ] No console errors in browser

### Integration ✅
- [ ] Backend and frontend communicate correctly
- [ ] Auth token works for protected routes
- [ ] Data flows from backend to Analytics page
- [ ] Issue creation saves to database
- [ ] Errors are handled gracefully

---

## 🐛 Known Issues / Acceptable Behaviors

1. **Confirm Dialog on Force Activation:**
   - One `confirm()` remains for forcing issue activation with incomplete nodes
   - **Status:** Acceptable - user confirmation is appropriate here

2. **React Router Warnings:**
   - Future flag warnings in tests
   - **Status:** Acceptable - will address in future upgrade

3. **Backend Warning:**
   - Unused `category` field in `StatsQueryParams`
   - **Status:** Acceptable - reserved for future filtering feature

---

## 📊 Success Criteria

**Phase 1 is considered PASSING if:**
- ✅ All backend tests pass (75/75)
- ✅ All frontend tests pass (94/94)
- ✅ Analytics page displays with real data
- ✅ CreateIssueModal works without prompts
- ✅ Error messages display in UI
- ✅ No critical bugs
- ✅ No regressions

**Quality Score Target:** **85/100 (B grade)**

---

## 🚀 Next Steps After Passing

If all tests pass, we're ready for:
1. **Commit Phase 1 changes**
2. **Begin Phase 2: Core UX Improvements**
3. **Target: 90/100 (A-)**

---

## 📝 Test Results Log

**Tester:** ________________
**Date:** ________________
**Time:** ________________

| Test | Status | Notes |
|------|--------|-------|
| Backend Stats | ⬜ Pass / ⬜ Fail | |
| Backend Sessions | ⬜ Pass / ⬜ Fail | |
| Analytics Page | ⬜ Pass / ⬜ Fail | |
| CreateIssueModal | ⬜ Pass / ⬜ Fail | |
| Error Messages | ⬜ Pass / ⬜ Fail | |
| Regression Tests | ⬜ Pass / ⬜ Fail | |

**Overall Status:** ⬜ PASS / ⬜ FAIL

**Issues Found:**
-

**Recommendations:**
-

---

**End of Phase 1 Testing Guide**
