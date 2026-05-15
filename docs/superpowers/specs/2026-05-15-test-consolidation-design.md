# Test Consolidation Design Document

**Date**: May 15, 2026  
**Objective**: Consolidate redundant Playwright tests while maintaining 100% coverage  
**Current State**: 39 tests across 27 files, 6,487 lines  
**Goal**: Reduce test count by 8-12 tests, eliminate redundancy

## Executive Summary

The Playwright test suite contains clear redundancies that can be safely eliminated:

1. **Password validation** tested identically across 3 flows (registration, invitation, password reset)
2. **Form field validation** patterns repeated throughout registration and settings tests
3. **Profile update tests** that can be parameterized
4. **Single-test files** that should be merged into appropriate feature files

No coverage will be lost—only test organization will improve.

## Current Test Inventory

### By Feature (39 tests total)

| Feature | File | Tests | Status |
|---------|------|-------|--------|
| Company Registration | 01-register_company.spec.ts | 15 | High redundancy (field validation) |
| Login | 02-login_admin.spec.ts | 2 | OK |
| User Invitations | 03-invite_user.spec.ts | 2 | OK |
| Invitation Acceptance | 04-accept_invitation.spec.ts | 7 | High redundancy (password) |
| Logs Management | 05-logs_management.spec.ts | 3 | OK |
| Template Designer | 06-template_designer.spec.ts | 2 | OK |
| User Admin Templates | 07-user_admin_templates.spec.ts | 2 | OK |
| Settings & Password | 08-settings_password.spec.ts | 6 | Medium redundancy (profile updates, password) |
| Google OAuth | 12-oauth-google.spec.ts | 1 | Can merge into auth suite |
| Google OAuth Unlink | 13-oauth-unlink.spec.ts | 1 | Can merge into auth suite |
| Branch Management | 15-branch_management.spec.ts | 1 | Single test, can merge |
| Hierarchical Roles | 16-hierarchical_roles.spec.ts | 1 | Single test, can merge |
| Profile Picture | 22-profile_picture.spec.ts | 1 | Can merge into settings |
| Others | 14, 16, 17-27 | ~10 | OK |

### Redundancy Map

**HIGH PRIORITY - Password Validation (Eliminate 5+ duplicate tests)**

```
Password Rules Testing Locations:
├─ 01-register_company.spec.ts
│  ├─ register_company_invalid_empty_password
│  ├─ register_company_invalid_empty_confirm_password
│  └─ register_company_invalid_password_mismatch
├─ 04-accept_invitation.spec.ts
│  ├─ accept_invitation_invalid_password_requirements
│  └─ accept_invitation_invalid_password_mismatch
└─ 08-settings_password.spec.ts
   ├─ reset_password_weak_password
   └─ reset_password_mismatch
```

**All test the same business rules:**
- Min 8 characters
- Uppercase required
- Lowercase required
- Number required
- Special character required
- Passwords must match

**Solution**: Create `validatePasswordField()` helper that tests all rules once, use across 3 files.

**MEDIUM PRIORITY - Form Field Validation (Eliminate 3-5 tests)**

```
Pattern: register_company_invalid_empty_*
├─ empty_company_name
├─ empty_address
├─ empty_first_name
├─ empty_last_name
├─ empty_email
└─ email_format
```

**Solution**: Create `testFieldValidation()` helper with parameterization.

**MEDIUM PRIORITY - Profile Updates (Eliminate 2-3 tests)**

```
08-settings_password.spec.ts
├─ update_first_name
├─ update_last_name
├─ update_both_names
├─ empty_first_name_validation
└─ empty_last_name_validation
```

**Solution**: Parameterize into matrix test.

**LOW PRIORITY - Single-Test Files (Merge 4-5 tests)**

Files with only 1 test should be consolidated:
- 12-oauth-google.spec.ts → Merge into login/auth suite
- 13-oauth-unlink.spec.ts → Merge into auth suite
- 15-branch_management.spec.ts → Merge into branch tests
- 16-hierarchical_roles.spec.ts → Merge into auth/admin tests
- 22-profile_picture.spec.ts → Merge into settings tests

## Implementation Strategy

### Phase 1: Create Shared Validators (No Changes to Tests)

Create two new utility files with helpers:

**`tests/shared-validators.ts`**
```typescript
// Validates password field across all contexts
export async function validatePasswordField(
  page: Page,
  testCase: 'weak' | 'mismatch' | 'empty',
  options?: { context: 'registration' | 'invitation' | 'reset' }
): Promise<void>

// Validates generic form fields
export async function testFieldValidation(
  page: Page,
  fieldName: string,
  validValue: string,
  invalidScenarios: Array<{ value?: string; scenario: string }>
): Promise<void>
```

### Phase 2: Consolidate Password Tests

**File: 01-register_company.spec.ts**
- Keep: `register_company`, password mismatch test (specific to registration context)
- Refactor: `register_company_invalid_empty_password` → call `validatePasswordField(page, 'empty')`
- Refactor: `register_company_invalid_password_mismatch` → call `validatePasswordField(page, 'mismatch')`

**File: 04-accept_invitation.spec.ts**
- Keep: Flow validation tests
- Remove: `accept_invitation_invalid_password_requirements` (duplicate)
- Remove: `accept_invitation_invalid_password_mismatch` (duplicate)
- Add: Single test calling `validatePasswordField()` in invitation context

**File: 08-settings_password.spec.ts**
- Remove: `reset_password_weak_password` (covered by validator)
- Remove: `reset_password_mismatch` (covered by validator)
- Add: Single password validation test calling helper

### Phase 3: Consolidate Form Field Tests

**File: 01-register_company.spec.ts**
- Current: 8 separate tests for empty fields + 1 email format
- Refactor: Parameterize into 2 tests:
  - `registration_validates_required_fields()` - tests all empty field scenarios
  - `registration_validates_email_format()` - tests email validation

### Phase 4: Consolidate Profile Updates

**File: 08-settings_password.spec.ts**
- Current: 5 separate tests
- Refactor: 2 parameterized tests:
  - `update_profile_fields()` - with matrix of [firstName, lastName, both]
  - `profile_field_validation()` - with matrix of empty field scenarios

### Phase 5: Merge Single-Test Files

1. **12-oauth-google.spec.ts** → Move `oauth_google_link_and_login` into login test suite
2. **13-oauth-unlink.spec.ts** → Move `oauth_google_unlink_and_attempt_signin` into login suite
3. **15-branch_management.spec.ts** → Move `create_and_list_branches` into branch operations tests
4. **16-hierarchical_roles.spec.ts** → Move `hierarchical_access_control` into auth/admin tests
5. **22-profile_picture.spec.ts** → Move `upload_and_persist_profile_picture` into settings tests

(Files 14-template_versioning and 26-internal-admin-security-log are larger and will be reviewed for consolidation separately)

## Coverage Verification

Each consolidated test will:
1. Preserve ALL original assertions
2. Preserve ALL original edge cases
3. Add comments explaining the consolidation
4. Use helper functions that are explicit and debuggable

Test execution flow remains identical—only organization changes.

## Success Criteria

- ✅ Test count reduced from 39 to 25-27 (30% reduction)
- ✅ All 39 original assertions preserved
- ✅ No new test failures introduced
- ✅ All tests pass locally and in CI
- ✅ Code coverage ≥ 95% maintained

## Risk Mitigation

- **Helpers make tests less explicit?** → Helpers will be simple, single-purpose functions with clear names
- **Harder to debug consolidated tests?** → Each consolidation adds comments explaining original test intent
- **Breaking existing test infrastructure?** → No changes to global-setup, utils, or core test utilities
- **CI pipeline impact?** → Test execution parallelism unchanged; runtime should improve (~5% fewer tests)

## Timeline

- Phase 1 (Helpers): 30 minutes
- Phase 2 (Password): 20 minutes
- Phase 3 (Form fields): 30 minutes
- Phase 4 (Profile): 15 minutes
- Phase 5 (Merge files): 20 minutes
- Verification & fix: 15 minutes
- **Total: ~2 hours**

## Files to Modify

### New Files
- `tests/shared-validators.ts`
- `tests/shared-form-helpers.ts`

### Modified Files
- `tests/01-register_company.spec.ts` (remove 4-5 tests)
- `tests/04-accept_invitation.spec.ts` (remove 2 tests)
- `tests/08-settings_password.spec.ts` (remove 3-4 tests, consolidate into 2-3)
- `tests/02-login_admin.spec.ts` (add merged OAuth tests)
- `tests/03-invite_user.spec.ts` (add branch management test)
- `tests/07-user_admin_templates.spec.ts` (add hierarchical roles test)
- `tests/[settings-related].spec.ts` (add profile picture test)

### Files to Delete
- `tests/12-oauth-google.spec.ts`
- `tests/13-oauth-unlink.spec.ts`
- `tests/15-branch_management.spec.ts`
- `tests/16-hierarchical_roles.spec.ts`
- `tests/22-profile_picture.spec.ts`

(14-template_versioning and 26-internal-admin-security-log will be evaluated separately)

## Next Steps

1. ✅ Review and approve this design
2. Implement Phase 1: Create shared validators
3. Implement Phase 2-5: Consolidations
4. Run full test suite
5. Verify no coverage loss
6. Create PR with consolidation changes
