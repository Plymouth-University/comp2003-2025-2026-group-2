# LogSmart - Project Documentation

## Overview

LogSmart is a comprehensive log management system and time tracker built with a Rust/Axum backend and SvelteKit frontend. The application enables companies to manage employee attendance, create customizable log templates, and generate reports. It is primarily aimed at the hospitality industry allowing head office to create log templates, set schedules and employees to fill out required logs digitally and securely.

### Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust, Axum 0.8.7 |
| Frontend | Svelte 5, SvelteKit |
| Auth DB | PostgreSQL |
| Log DB | MongoDB |
| Authentication | JWT, WebAuthn/Passkeys, Google OAuth |
| Deployment | Cloudflare Pages (frontend), Docker (backend) |

---

## Architecture

### Project Structure

```
SourceCode/
├── back-end/                    # Rust/Axum API server
│   ├── src/
│   │   ├── main.rs             # Application entry point
│   │   ├── lib.rs              # Library root, exports modules
│   │   ├── db.rs               # PostgreSQL models and queries
│   │   ├── logs_db.rs          # MongoDB models and queries
│   │   ├── dto.rs              # Data Transfer Objects
│   │   ├── auth.rs             # Authentication utilities
│   │   ├── security.rs         # Security utilities
│   │   ├── jwt_manager.rs      # JWT token management
│   │   ├── rate_limit.rs       # Rate limiting logic
│   │   ├── middleware.rs       # Axum middleware
│   │   ├── email.rs            # Email sending logic
│   │   ├── llm.rs              # LLM integration
│   │   ├── metrics.rs          # Application metrics
│   │   ├── handlers/           # HTTP request handlers
│   │   │   ├── auth_handlers.rs
│   │   │   ├── user_handlers.rs
│   │   │   ├── branch_handlers.rs
│   │   │   ├── clock_handlers.rs
│   │   │   ├── template_handlers.rs
│   │   │   ├── log_entry_handlers.rs
│   │   │   ├── invitation_handlers.rs
│   │   │   ├── passkey_handlers.rs
│   │   │   ├── oauth_handlers.rs
│   │   │   ├── health_handlers.rs
│   │   │   └── llm_handlers.rs
│   │   └── services/           # Business logic layer
│   │       ├── auth_service.rs
│   │       ├── user_service.rs
│   │       ├── invitation_service.rs
│   │       ├── template_service.rs
│   │       ├── clock_service.rs
│   │       ├── log_entry_service.rs
│   │       └── oauth_service.rs
│   ├── migrations/            # SQLx database migrations
│   ├── tests/                 # Integration and unit tests
│   ├── Cargo.toml
│   └── docker-compose.yml
│
└── front-end/
    └── logsmart/               # SvelteKit application
        ├── src/
        │   ├── routes/        # SvelteKit routes
        │   │   ├── login/
        │   │   ├── register-company/
        │   │   ├── accept-invitation/
        │   │   ├── reset-password/
        │   │   ├── (authenticated)/     # Protected routes
        │   │   │   ├── dashboard/
        │   │   │   ├── users-admin/
        │   │   │   ├── branches/
        │   │   │   ├── attendance-admin/
        │   │   │   ├── templates-dashboard/
        │   │   │   ├── template-designer/
        │   │   │   ├── logs-list/
        │   │   │   ├── log-template/
        │   │   │   ├── reports/
        │   │   │   ├── company-settings/
        │   │   │   └── settings/
        │   │   └── (internal-admin-authenticated)/
        │   │       └── admin-dashboard/
        │   ├── lib/
        │   │   ├── api.ts     # OpenAPI client
        │   │   ├── stores/   # Svelte stores
        │   │   └── components/  # Reusable components
        │   ├── app.html
        │   └── app.css
        └── package.json
```

---

## Database Schema

### PostgreSQL (Authentication & Organization)

#### Users Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| email | VARCHAR(255) | Unique user email |
| first_name | VARCHAR(100) | User's first name |
| last_name | VARCHAR(100) | User's last name |
| password_hash | VARCHAR(255) | Argon2 hashed password (nullable) |
| company_id | UUID | FK to companies (nullable for super admins) |
| branch_id | UUID | FK to branches (nullable) |
| role | user_role | Enum: logsmart_admin, company_manager, branch_manager, staff |
| oauth_provider | VARCHAR(50) | OAuth provider (google, etc.) |
| oauth_subject | VARCHAR(255) | OAuth provider subject ID |
| profile_picture_id | VARCHAR(255) | Profile picture storage ID |
| created_at | TIMESTAMP | Account creation time |
| deleted_at | TIMESTAMP | Soft delete timestamp |

**User Roles Hierarchy:**
1. `logsmart_admin` - Super admin with system-wide access
2. `company_manager` - Manages entire company
3. `branch_manager` - Manages specific branch
4. `staff` - Regular employee

#### Companies Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| name | VARCHAR(255) | Company name |
| address | TEXT | Company address |
| created_at | TIMESTAMP | Creation time |

#### Branches Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| company_id | UUID | FK to companies |
| name | VARCHAR(255) | Branch name |
| address | TEXT | Branch address |
| created_at | TIMESTAMP | Creation time |

#### Invitations Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| company_id | UUID | FK to companies |
| email | VARCHAR(255) | Invitee email |
| token | VARCHAR(255) | Unique invitation token |
| role | user_role | Assigned role |
| branch_id | UUID | Optional branch assignment |
| created_at | TIMESTAMP | Invitation creation |
| expires_at | TIMESTAMP | Expiration time |
| accepted_at | TIMESTAMP | Acceptance timestamp |
| cancelled_at | TIMESTAMP | Cancellation timestamp |

#### Passkeys Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | FK to users |
| credential_id | VARCHAR(255) | WebAuthn credential ID (Base64URL) |
| public_key | TEXT | WebAuthn public key (JSON) |
| counter | BIGINT | Authentication counter |
| name | VARCHAR(255) | Passkey name (e.g., "MacBook Pro") |
| created_at | TIMESTAMP | Creation time |
| last_used_at | TIMESTAMP | Last authentication time |

#### Clock Events Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | FK to users |
| company_id | UUID | FK to companies |
| clock_in | TIMESTAMP | Clock in time |
| clock_out | TIMESTAMP | Clock out time (nullable) |
| created_at | TIMESTAMP | Record creation time |

#### Security Logs Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| event_type | VARCHAR(100) | Event type (login, logout, etc.) |
| user_id | UUID | FK to users (nullable) |
| email | VARCHAR(255) | User email if known |
| ip_address | INET | Client IP address |
| user_agent | TEXT | Client user agent |
| details | JSONB | Additional event details |
| success | BOOLEAN | Whether action succeeded |
| created_at | TIMESTAMP | Event time |

#### Password Resets Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | FK to users |
| token | VARCHAR(255) | Reset token |
| created_at | TIMESTAMP | Token creation |
| expires_at | TIMESTAMP | Expiration time |
| used_at | TIMESTAMP | When token was used |

#### Branch Deletion Tokens Table

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | FK to users requesting deletion |
| branch_id | UUID | FK to branches being deleted |
| token | VARCHAR(255) | Confirmation token |
| created_at | TIMESTAMP | Token creation |
| expires_at | TIMESTAMP | Expiration time |
| used_at | TIMESTAMP | When confirmed |

### MongoDB (Log Templates & Entries)

#### Templates Collection

```typescript
interface TemplateDocument {
  _id: ObjectId;
  template_name: string;
  template_layout: TemplateField[];
  company_id: string;
  branch_id: string | null;
  created_at: Date;
  updated_at: Date;
  schedule: {
    frequency: 'Daily' | 'Weekly' | 'Monthly' | 'Yearly';
    days_of_week?: number[];      // 0-6 for weekly
    day_of_week?: number;         // for weekly schedule
    day_of_month?: number;        // for monthly schedule
    month_of_year?: number;       // for yearly schedule
  };
  created_by: string;             // User UUID
  version: number;
  version_name?: string;
}

interface TemplateField {
  field_type: string;             // 'text', 'checkbox', 'temperature', etc.
  position: { x: number; y: number };
  props: {
    text?: string;
    size?: string;
    weight?: string;
    value?: string;
    min?: number;
    max?: number;
    unit?: string;
    selected?: string;
    options?: string[];
    editable?: boolean;
    placeholder?: string;
    font_family?: string;
    text_decoration?: string;
    color?: string;
    required?: boolean;
    max_length?: number;
    min_length?: number;
    input_type?: string;
  };
}
```

#### Log Entries Collection

```typescript
interface LogEntry {
  _id: ObjectId;
  template_id: ObjectId;
  template_name: string;
  user_id: string;
  company_id: string;
  branch_id: string | null;
  data: Record<string, any>;       // Field values submitted
  due_date: Date;
  submitted_at: Date;
  submitted: boolean;
  created_at: Date;
  updated_at: Date;
}
```

---

## API Endpoints

### Authentication Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/register` | Register new company (first admin) |
| POST | `/auth/login` | Email/password login |
| POST | `/auth/verify` | Verify JWT token |
| GET | `/auth/google/initiate` | Start Google OAuth flow |
| GET | `/auth/google/callback` | Google OAuth callback |
| POST | `/auth/google/link` | Link Google account to existing user |
| POST | `/auth/google/link/confirm` | Confirm Google account linking |
| DELETE | `/auth/google/unlink` | Unlink Google account |
| POST | `/auth/invitations/accept` | Accept invitation and create account |
| PUT | `/auth/invitations/cancel` | Cancel pending invitation |
| GET | `/auth/invitations/details` | Get invitation details by token |
| POST | `/auth/password/request-reset` | Request password reset email |
| POST | `/auth/password/reset` | Reset password with token |
| GET | `/auth/me` | Get current authenticated user |
| POST | `/auth/passkey/register/start` | Start passkey registration |
| POST | `/auth/passkey/register/finish` | Complete passkey registration |
| POST | `/auth/passkey/login/start` | Start passkey login (non-discoverable) |
| POST | `/auth/passkey/login/finish` | Complete passkey login |
| POST | `/auth/passkey/login/discoverable/start` | Start discoverable passkey login |
| POST | `/auth/passkey/login/discoverable/finish` | Complete discoverable passkey login |
| GET | `/auth/passkeys` | List user's passkeys |
| DELETE | `/auth/passkeys/{passkey_id}` | Delete a passkey |
| PUT | `/auth/profile` | Update user profile |
| POST | `/auth/invitations/send` | Send invitation to join company |
| GET | `/auth/invitations/pending` | Get pending invitations |
| GET | `/auth/company/members` | Get company members |
| POST | `/auth/profile-picture` | Upload profile picture |
| DELETE | `/auth/profile-picture` | Delete profile picture |
| GET | `/auth/profile-picture/{user_id}` | Get user profile picture |

### Company & Branch Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/auth/company/branches` | Create new branch |
| GET | `/auth/company/branches` | List company branches |
| PUT | `/auth/company/branches` | Update branch |
| POST | `/auth/company/branches/request-deletion` | Request branch deletion |
| POST | `/auth/company/branches/confirm-deletion` | Confirm branch deletion |

### Admin Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| PUT | `/auth/admin/update-member` | Admin update user profile/role |
| DELETE | `/auth/admin/remove-member` | Admin remove user from company |

### Template Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/logs/templates` | Create new template |
| GET | `/logs/templates` | Get template by ID |
| GET | `/logs/templates/all` | Get all company templates |
| PUT | `/logs/templates/update` | Update template |
| PUT | `/logs/templates/rename` | Rename template |
| DELETE | `/logs/templates` | Delete template |
| GET | `/logs/templates/versions` | Get template version history |
| POST | `/logs/templates/versions/restore` | Restore template version |

### Log Entry Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/logs/entries/due` | Get forms due today |
| POST | `/logs/entries` | Create new log entry |
| GET | `/logs/entries` | Get user's log entries |
| GET | `/logs/admin/entries` | Admin get all company entries |
| GET | `/logs/entries/{entry_id}` | Get specific entry |
| PUT | `/logs/entries/{entry_id}` | Update entry |
| DELETE | `/logs/entries/{entry_id}` | Delete entry |
| POST | `/logs/entries/{entry_id}/submit` | Submit entry |
| POST | `/logs/entries/{entry_id}/unsubmit` | Unsubmit entry |

### Clock In/Out Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/clock/in` | Clock in |
| POST | `/clock/out` | Clock out |
| GET | `/clock/status` | Get current clock status |
| GET | `/clock/company` | Get company clock events |

### LLM Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/llm/generate-layout` | Generate template layout using LLM |

### Health Check Routes

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Basic health check |
| GET | `/health/database` | Database health metrics |
| GET | `/health/slow-queries` | Get slow queries |
| GET | `/health/index-usage` | Get index usage stats |
| GET | `/health/table-sizes` | Get table sizes |

---

## Authentication & Security

### JWT Authentication

- Tokens are signed with `JWT_SECRET` environment variable
- Default expiration: 24 hours
- Token contains: user_id, email, role, company_id
- Refresh tokens not implemented - users must re-login

### Password Security

- Password hashing: Argon2id
- Minimum password requirements: 8+ characters
- Password reset tokens expire after 1 hour

### WebAuthn/Passkeys

- Supports discoverable and non-discoverable credentials
- Session challenges expire after 5 minutes
- Credential counter tracking to prevent replay attacks

### Google OAuth

- OAuth 2.0 with OpenID Connect
- Supports account linking (add Google to existing account)
- Stores provider, subject, and profile picture

### Rate Limiting

- Default: 100 requests per minute per IP
- Uses in-memory token bucket algorithm (governor crate)
- Clean-up task runs every 60 seconds

### Security Logging

All security-relevant events are logged:
- Login attempts (success/failure)
- Password resets
- Account creation/deletion
- Permission changes
- OAuth account links/unlinks

---

## Frontend Pages

### Public Routes

| Route | Description |
|-------|-------------|
| `/` | Landing page |
| `/login` | Login page with email/password, Google OAuth, passkey options |
| `/register-company` | Company registration (first admin) |
| `/accept-invitation` | Accept invitation and create account |
| `/reset-password` | Password reset page |
| `/terms` | Terms of service |
| `/privacy` | Privacy policy |
| `/cookie-policy` | Cookie policy |
| `/contact` | Contact page |

### Authenticated Routes

| Route | Description |
|-------|-------------|
| `/dashboard` | User dashboard with quick stats |
| `/users-admin` | User management (invite, edit roles) |
| `/branches` | Branch management |
| `/attendance-admin` | View all clock in/out events |
| `/templates-dashboard` | List all templates |
| `/template-designer` | Visual template editor |
| `/logs-list` | View all log entries |
| `/log-template/{id}` | Fill out a log form |
| `/reports` | Generate reports |
| `/company-settings` | Company settings (managers only) |
| `/settings` | User personal settings |

### Internal Admin Routes

| Route | Description |
|-------|-------------|
| `/admin-dashboard` | System-wide admin dashboard (logsmart_admin only) |

---

## Frontend Components

### Reusable Components (`src/lib/components/`)

| Component | Description |
|-----------|-------------|
| `ClockInOut.svelte` | Clock in/out button with status |
| `user_text_input.svelte` | Text input with validation |
| `user_checkbox.svelte` | Checkbox component |
| `user_text_label.svelte` | Text label component |
| `temperature_picker.svelte` | Temperature input with unit |
| `ProfilePictureUploader.svelte` | Profile picture upload/crop |
| `CookieConsent.svelte` | Cookie consent banner |
| `pwa_install_prompt.svelte` | PWA install prompt |
| `user_dropdown.svelte` | Dropdown select |

### Template Designer Components

| Component | Description |
|-----------|-------------|
| `DesignCanvas.svelte` | Main canvas for placing fields |
| `ComponentsPalette.svelte` | Available field types |
| `PropertiesPanel.svelte` | Field properties editor |
| `CanvasItem.svelte` | Individual field on canvas |
| `TemplatesSidebar.svelte` | Template list sidebar |
| `AiGeneratorSidebar.svelte` | AI layout generation |
| `VersionHistoryModal.svelte` | Template version history |

---

## Configuration

### Environment Variables

#### Backend Required

| Variable | Description |
|----------|-------------|
| `POSTGRES_HOST` | PostgreSQL host (default: localhost) |
| `POSTGRES_PORT` | PostgreSQL port (default: 5432) |
| `POSTGRES_USER` | PostgreSQL username |
| `POSTGRES_PASSWORD` | PostgreSQL password |
| `POSTGRES_DB` | Database name (default: logsmartdb) |
| `JWT_SECRET` | JWT signing secret |
| `SMTP_USERNAME` | Email SMTP username |
| `SMTP_PASSWORD` | Email SMTP password |
| `GOOGLE_CLIENT_ID` | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret |
| `GOOGLE_REDIRECT_URI` | OAuth redirect URI |
| `GOOGLE_ISSUER_URL` | Google issuer URL |
| `RP_ID` | WebAuthn Relying Party ID (default: localhost) |
| `RP_ORIGIN` | WebAuthn origin (default: http://localhost:5173) |
| `MONGODB_URI` | MongoDB connection URI |

#### Backend Optional

| Variable | Description |
|----------|-------------|
| `LOG_FORMAT` | Log format: "text" or "json" (default: text) |
| `OPENROUTER_API_KEY` | OpenRouter API key for LLM features |
| `OPENROUTER_MODEL` | OpenRouter model (default: `openrouter/free`) |
| `OPENROUTER_URL` | OpenRouter chat completions URL |

#### Frontend

| Variable | Description |
|----------|-------------|
| `PUBLIC_API_URL` | Backend API URL (default: /api proxy) |

---

## Development

### Prerequisites

- Docker and Docker Compose
- Rust (latest stable)
- Bun (for frontend)
- Node.js (for Playwright)

### Running Backend with Docker Compose

The recommended way to run the backend is using Docker Compose, which starts PostgreSQL, MongoDB, and the API server:

```bash
cd back-end
docker compose up -d
```

This starts:
- PostgreSQL on port 5432
- MongoDB on port 27017
- API server on port 6767

To view logs:
```bash
docker compose logs -f
```

To stop:
```bash
docker compose down
```

### Running Backend Locally (Alternative)

If you prefer to run the backend directly (requires PostgreSQL and MongoDB already running):

```bash
cd back-end
cargo run
```

Server runs on `http://localhost:6767`

### Running Frontend

```bash
cd front-end/logsmart
bun run dev
```

Dev server runs on `http://localhost:5173`

### Database Migrations

Migrations are in `back-end/migrations/` and run automatically on startup via SQLx.

### Generating API Documentation

The OpenAPI specification is generated from the backend and used to create TypeScript types for the frontend:

```bash
cd back-end
cargo run --bin gen_spec > ../front-end/logsmart/openapi.json
```

Then regenerate the TypeScript types:

```bash
cd front-end/logsmart
bun run gen:api
```

This updates `src/lib/api-types.d.ts` with types matching the current API.

Swagger UI is also available at `http://localhost:6767/swagger-ui` when the backend is running.

### Running Playwright Tests

The project includes end-to-end tests using Playwright. These tests require Docker to be running (for PostgreSQL, MongoDB, and the backend API).

**Prerequisites:**
1. Ensure Docker is running
2. Start the backend services:
   ```bash
   cd back-end
   docker compose up -d
   ```
3. Install Playwright browsers (first time only):
   ```bash
   cd front-end/logsmart
   npx playwright install chromium
   ```

**Run all tests:**
```bash
cd front-end/logsmart
npx playwright test
```

**Run specific test file:**
```bash
npx playwright test tests/login.spec.ts
```

**Run tests with UI:**
```bash
npx playwright test --ui
```

**Run tests in headed mode (see browser):**
```bash
npx playwright test --headed
```

The Playwright configuration:
- Tests are located in `front-end/logsmart/tests/`
- The frontend dev server runs automatically on port 5173 during tests
- API calls are proxied to the backend at localhost:6767

---

## Key Services

### Backend Services (`src/services/`)

| Service | Responsibility |
|---------|---------------|
| `auth_service.rs` | User authentication, registration, password management |
| `user_service.rs` | User CRUD, profile management |
| `invitation_service.rs` | Invitation creation, acceptance, cancellation |
| `template_service.rs` | Template CRUD, version management |
| `clock_service.rs` | Clock in/out logic |
| `log_entry_service.rs` | Log entry creation, submission |
| `oauth_service.rs` | Google OAuth flow handling |

---

## Testing

Backend tests are in `back-end/tests/`:
- `auth_tests.rs` - Authentication flow tests
- `handlers_tests.rs` - Handler unit tests
- `service_tests.rs` - Service layer tests
- `db_tests.rs` - Database tests
- `api_integration_tests.rs` - Full API integration tests
- `security_tests.rs` - Security feature tests
- `rate_limit_tests.rs` - Rate limiting tests

Run tests:
```bash
cd back-end
cargo test
```

---

## Deployment

### Backend
- Docker container with PostgreSQL and MongoDB
- Uses `docker-compose.yml` for orchestration

### Frontend
- Deployed to Cloudflare Pages
- SvelteKit with `@sveltejs/adapter-cloudflare`
- PWA support with `@vite-pwa/sveltekit`

---

## Common Development Tasks

### Adding a New Template Field Type

1. **Backend**: Add field type to `TemplateField` in `logs_db.rs`
2. **Frontend**: Add component in `template-designer/`
3. **Database**: New fields don't require migration (MongoDB is schema-less)

### Adding a New API Endpoint

1. Define DTO in `dto.rs`
2. Add handler in appropriate `handlers/*.rs`
3. Add service method in appropriate `services/*.rs`
4. Add database queries in `db.rs` or `logs_db.rs`
5. Register route in `main.rs`
6. Update OpenAPI docs in `api_docs.rs`

### Adding a New User Role

1. Add variant to `UserRole` enum in `db.rs`
2. Add SQL enum in migration
3. Update role checks in handlers/services
4. Update frontend role handling

---

## Notes for AI/LLM Context

- This is a multi-tenant system where users belong to companies
- Branches are optional - users can be company-wide (HQ staff)
- Templates are company-scoped, optionally branch-specific
- Log entries are user-scoped but company-viewable
- Clock events are company-scoped for admin viewing
- Super admins (`logsmart_admin`) can access all companies but not regular user features
- MongoDB is used for flexible schema templates and form data
- PostgreSQL is used for relational data with strict schema

---

## Role Permissions Matrix

### User Roles Hierarchy

| Role | Description |
|------|-------------|
| `logsmart_admin` | Super admin - system-wide access to all companies |
| `company_manager` | Full access to their company (all branches) |
| `branch_manager` | Access to their specific branch only |
| `staff` | Regular employee - can fill logs and clock in/out |

### Permission Extractors

The backend uses Axum extractors for role-based access control:

| Extractor | Access |
|-----------|--------|
| `LogSmartAdminUser` | logsmart_admin only |
| `ManageCompanyUser` | company_manager, logsmart_admin |
| `BranchManagerUser` | branch_manager, company_manager, logsmart_admin |
| `ReadCompanyUser` | company_manager + readonly_hq (staff without branch) |
| `ReadBranchUser` | branch_manager + readonly_hq + company_manager + logsmart_admin |
| `AnyAuthUser` | staff, branch_manager, company_manager, logsmart_admin |

### Feature Access Matrix

| Feature | logsmart_admin | company_manager | branch_manager | readonly_hq | staff |
|---------|----------------|-----------------|---------------|------------|-------|
| **Authentication** |
| Login (email/password) | ✓ | ✓ | ✓ | ✓ | ✓ |
| Login (Google OAuth) | ✓ | ✓ | ✓ | ✓ | ✓ |
| Login (Passkey) | ✓ | ✓ | ✓ | ✓ | ✓ |
| Register company | Requires no auth |
| **User Management** |
| View all company members | ✓ | ✓ | Own branch | ✓ | - |
| Invite users | ✓ | ✓ | Own branch | - | - |
| Cancel invitations | ✓ | ✓ | Own branch | - | - |
| Update member profile/role | ✓ | ✓ | Own branch | - | - |
| Remove member | ✓ | ✓ | Own branch | - | - |
| **Branch Management** |
| Create branch | ✓ | ✓ | - | - | - |
| Update branch | ✓ | ✓ | Own branch | - | - |
| Delete branch | ✓ | ✓ | - | - | - |
| View branches | ✓ | ✓ | ✓ | ✓ | - |
| **Templates** |
| Create template | ✓ | ✓ | Own branch | - | - |
| Edit template | ✓ | ✓ | Own branch | - | - |
| Delete template | ✓ | ✓ | Own branch | - | - |
| View company templates | ✓ | ✓ | Own branch | ✓ | - |
| View branch templates | ✓ | ✓ | Own branch | ✓ | - |
| Restore template version | ✓ | ✓ | Own branch | - | - |
| **Log Entries** |
| Create log entry | ✓ | ✓ | ✓ | - | ✓ |
| Submit log entry | ✓ | ✓ | ✓ | - | ✓ |
| View own entries | ✓ | ✓ | ✓ | ✓ | ✓ |
| View all company entries | ✓ | ✓ | Own branch | ✓ | - |
| Update any entry | ✓ | ✓ | Own branch | - | - |
| Delete any entry | ✓ | ✓ | Own branch | - | - |
| **Attendance** |
| Clock in/out | ✓ | ✓ | ✓ | ✓ | ✓ |
| View own clock status | ✓ | ✓ | ✓ | ✓ | ✓ |
| View company attendance | ✓ | ✓ | Own branch | ✓ | - |
| **Reports** |
| Generate reports | ✓ | ✓ | Own branch | ✓ | - |
| **Company Settings** |
| View settings | ✓ | ✓ | - | - | - |
| Update settings | ✓ | ✓ | - | - | - |
| **Admin Dashboard** |
| System-wide admin panel | ✓ | - | - | - | - |
| **Profile** |
| Update own profile | ✓ | ✓ | ✓ | ✓ | ✓ |
| Upload profile picture | ✓ | ✓ | ✓ | ✓ | ✓ |
| Add/remove passkeys | ✓ | ✓ | ✓ | ✓ | ✓ |
| Link/unlink Google | ✓ | ✓ | ✓ | ✓ | ✓ |
| Request password reset | Public |
| Reset password | Public |

### Permission Logic

From `db.rs` - helper methods on `UserRecord`:

```rust
// Can manage entire company (company_manager + logsmart_admin)
user.can_manage_company() -> bool

// Can manage branches (branch_manager + can_manage_company)
user.can_manage_branch() -> bool

// Is staff without branch assignment (HQ staff)
user.is_readonly_hq() -> bool

// Can read/manage branch or is readonly_hq
user.can_read_manage_branch() -> bool
```

### Special Cases

- **HQ Staff** (`staff` role without `branch_id`): Can view company-wide data but cannot manage branches
- **Branch Manager**: Can only access data for their assigned branch
- **LogSmart Admin**: Can access all companies but cannot perform regular user actions (clock in, fill logs) - they are system admins
- **Invitations**: Branch managers can only invite users to their branch

---

## Git Conventions

### Branch Naming

Use descriptive prefixes for branch names:

| Prefix | Use Case |
|--------|----------|
| `feature/` | New features (e.g., `feature/add-user-invites`) |
| `fix/` | Bug fixes (e.g., `fix/login-redirect-issue`) |
| `refactor/` | Code refactoring |
| `docs/` | Documentation updates |
| `test/` | Adding or fixing tests |
| `chore/` | Maintenance tasks |

### Commit Message Style

Follow this format:

```
<Scope>: <Subject>

[Optional body]
```

**Examples:**

```
Front-End: Add user invitation modal

Implemented a modal component for inviting new users to the company.
Includes email validation and role selection.

Fix: Resolve clock-out not saving

Backend: Fix rate limiting middleware

Docs: Update API endpoint documentation
```

**Rules:**
- Subject line should be concise (max 50 characters)
- Use imperative mood ("Add" not "Added" or "Adds")
- Prefix with area: `Front-End:`, `Back-End:`, `Docs:`, `Fix:`, etc.
- Capitalize the first letter of the subject
- No period at the end of subject line
- Separate subject from body with a blank line

### GitHub Actions Workflows

The project includes CI/CD workflows:

**Test Workflow** (`back-end/.github/workflows/test.yml`):
- Runs Rust backend tests on push and pull requests
- Executes cargo test with all test suites

---

## Common Tasks Reference

### Adding a New Template Field Type

1. **Backend**: Add field type to `TemplateField` in `logs_db.rs`
2. **Frontend**: Add component in `template-designer/`
3. **Database**: New fields don't require migration (MongoDB is schema-less)

### Adding a New API Endpoint

1. Define DTO in `dto.rs`
2. Add handler in appropriate `handlers/*.rs`
3. Add service method in appropriate `services/*.rs`
4. Add database queries in `db.rs` or `logs_db.rs`
5. Register route in `main.rs`
6. Update OpenAPI docs in `api_docs.rs`
7. Regenerate API spec: `cargo run --bin gen_spec > ../front-end/logsmart/openapi.json`

### Adding a New User Role

1. Add variant to `UserRole` enum in `db.rs`
2. Add SQL enum in migration
3. Update role checks in handlers/services
4. Update frontend role handling
