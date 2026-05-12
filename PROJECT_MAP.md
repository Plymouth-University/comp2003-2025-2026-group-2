# LogSmart Project Map

This document serves as a technical deep-dive for agents and developers working on the LogSmart codebase. It maps out backend data models to their frontend usages, defines database boundaries, documents the API code generation chain, and outlines the testing inventory.

## 1. Data Model Cross-Reference (`dto.rs` -> Frontend Types)

All Data Transfer Objects (DTOs) are defined in `SourceCode/back-end/src/dto.rs` using Rust structs and Enums. The frontend consumes these automatically via OpenAPI (as `components['schemas']['ModelName']` within `src/lib/api-types.d.ts`).

Here are the primary DTOs and where they're typically consumed in the SvelteKit frontend:

*   **User & Auth**:
    *   `UserResponse`: Used in `attendance-admin`, `company-settings`, `logs-list`, `users-admin`, and `admin-dashboard`.
    *   `AuthResponse`, `LoginRequest`, `RegisterRequest`, `JwtVerifyResponse`: Used system-wide via `api.ts` types for session management.
    *   `PasswordResetResponse`, `ResetPasswordRequest`, `RequestPasswordResetRequest`: For account recovery.
*   **Companies & Branches**:
    *   `CompanyResponse`: Used in `company-settings`.
    *   `BranchDto`, `CreateBranchRequest`, `UpdateBranchRequest`, `ListBranchesResponse`: Used in branch management UI.
    *   `GetCompanyMembersResponse`: Used in `users-admin`.
*   **Invitations**:
    *   `InvitationResponse`: Used in `users-admin` UI.
    *   `GetPendingInvitationsResponse`, `InviteUserRequest`, `AcceptInvitationRequest`, `CancelInvitationRequest`: Handled in user administration views.
*   **Templates (MongoDB)**:
    *   `TemplateInfo`, `GetTemplateResponse`, `TemplateVersionInfo`: Used heavily in `template-designer` and `templates-dashboard`.
    *   `AddTemplateRequest`, `UpdateTemplateRequest`, `RenameTemplateRequest`, `DeleteTemplateRequest`.
    *   `Schedule`, `TemplateField`, `TemplateFieldProps`: Consumed directly as core configuration objects in `template-designer` components (e.g. `defaultTemplates.ts`).
*   **Logs & Submissions (MongoDB)**:
    *   `LogEntryResponse`: Used heavily in `logs-list`, `log-template`, and `reports`.
    *   `ListLogEntriesResponse`, `CreateLogEntryRequest`, `SubmitLogEntryResponse`.
    *   `DueFormInfo`, `DueFormsResponse`: Core objects for the `dashboard` and `logs-list` routing to indicate pending work.
*   **Time & Attendance (Postgres)**:
    *   `ClockEventResponse`, `ClockStatusResponse`: Used across the dashboard for time tracking status.
    *   `CompanyClockEventResponse`, `CompanyClockEventsResponse`: Used in `attendance-admin`.
*   **Reporting**:
    *   `ReportRunParams`, `ReportRunResponse`, `ListReportRunsResponse`, `CreateReportRunRequest`.
*   **Passkeys**:
    *   `PasskeyDto`, `ListPasskeysResponse`, `PasskeyRegistrationStartRequest`, `PasskeyAuthenticationStartRequest`, etc.

---

## 2. Database Routing

LogSmart uses a polyglot persistence architecture. Models and their corresponding repositories generally implement `FromRow` for Postgres or `Deserialize` for MongoDB.

### PostgreSQL (`SourceCode/back-end/src/db.rs`)
PostgreSQL acts as the primary relational store handling users, organization structure, auth, and attendance metrics.

*   `users` - User accounts, profiles, and credentials.
*   `companies` - Organizations/tenants boundaries.
*   `branches` - Physical or logical locations within companies.
*   `branch_deletion_tokens` - Secure verification steps for branch removal.
*   `invitations` - Pending user invitations and RBAC provisioning.
*   `clock_events` - Time tracking and attendance records.
*   `passkeys` - WebAuthn credentials.
*   `passkey_sessions` - WebAuthn authentication challenge sessions.
*   `password_resets` - Tokens for password recovery workflows.
*   `security_logs` - Audit trails and security-sensitive events.

### MongoDB (`SourceCode/back-end/src/logs_db.rs`)
MongoDB acts as the schema-less document store to handle highly dynamic form structures (templates) and their submitted instances (logs).

*   `templates` - The dynamic form definitions and rules created in the template designer.
*   `template_versions` - Immutable version history and auditability for templates.
*   `log_entries` - The submitted values corresponding to a template version.
*   `report_runs` - Asynchronous report execution records and output metadata.

---

## 3. The Codegen Chain

LogSmart enforces strict type safety across the application stack by generating OpenAPI specifications from Rust code, which are subsequently converted into TypeScript definitions on the frontend.

**Workflow Summary:**
1.  **Rust Backend:** DTOs in `src/dto.rs` derive `ToSchema`. Handlers are annotated with `#[utoipa::path(...)]`.
2.  **Spec Generation (`gen_spec.rs`):** A Rust binary collects annotations and dumps the OpenAPI JSON.
3.  **Frontend Generation (`openapi-typescript`):** A frontend process parses the JSON and outputs fully-typed TypeScript interfaces.

**Execution Path:**
```bash
# 1. Generate OpenAPI spec from Backend
cd SourceCode/back-end
cargo run --bin gen_spec > ../front-end/logsmart/openapi.json

# 2. Generate Frontend TypeScript definitions
cd ../front-end/logsmart
bun run gen:api
```

**File Dependencies:**
*   `SourceCode/back-end/src/bin/gen_spec.rs` -> The Rust generator binary.
*   `SourceCode/front-end/logsmart/openapi.json` -> The intermediary OpenAPI v3 JSON spec.
*   `SourceCode/front-end/logsmart/package.json` -> Contains the `gen:api` execution script (`openapi-typescript ...`).
*   `SourceCode/front-end/logsmart/src/lib/api-types.d.ts` -> The final TypeScript definitions consumed by `src/lib/api.ts`.

---

## 4. Testing Inventory (`SourceCode/back-end/tests/`)

The backend integration test suite relies on centralized factories and mocks to predictably isolate services and database layer logic. 

### Common Infrastructure (`tests/common/`)
*   **`config.rs`**: Handles environment variables and DB connection pooling for tests.
*   **`factories.rs`**: Object builders that agents should use to quickly provision database state:
    *   `UserFactory`
    *   `CompanyFactory`
    *   `InvitationFactory`
    *   `PasskeyFactory` & `PasskeySessionFactory`
    *   `SecurityLogFactory`
    *   `TemplateFactory` (MongoDB bound)
    *   `LogEntryFactory` (MongoDB bound)
*   **`mocks.rs`**: Mock implementations representing external system boundaries:
    *   `MockWebAuthnService`
    *   `MockEmailService`
    *   `MockOAuthService` & `MockOAuthStateStore`
    *   `MockRateLimiter`

### Integration Test Suites
When modifying components, tests in the following suites must be maintained/executed:
*   `api_integration_tests.rs`: High-level E2E API tests.
*   `auth_tests.rs` / `auth_unit_tests.rs`: Auth flows including credentials, JWTs, and Passkeys.
*   `clock_tests.rs`: Time tracking, concurrency, and attendance logic.
*   `db_tests.rs` / `db_model_tests.rs`: Specific SQL/NoSQL query correctness and schema validations.
*   `handlers_tests.rs`: Axum router integration.
*   `http_api_tests.rs`: Comprehensive suite for REST constraints.
*   `rate_limit_tests.rs`: Throttling validations.
*   `scheduling_tests.rs`: Test assertions around templates, form assignments, and dashboard 'Due' statuses.
*   `service_tests.rs` / `service_basic_tests.rs`: Standard service-level isolation tests.