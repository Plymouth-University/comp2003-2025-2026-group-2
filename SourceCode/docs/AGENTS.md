# LogSmart - Agent Guidelines

## Context & Environment
- **Stack**: Rust 2024/Axum (port 6767), SvelteKit 5 (port 5173), PostgreSQL, MongoDB
- **Nix Env**: Run `direnv exec . <command>` in `SourceCode/` or `SourceCode/back-end/` for Rust toolchain + cross-compile env vars.

## Architecture Boundaries
- **Backend (`SourceCode/back-end/`)**: 
  - `src/db.rs`: PostgreSQL (SQLx) models.
  - `src/logs_db.rs`: MongoDB models.
  - `src/dto.rs`: DTOs (derive `Serialize, Deserialize, ToSchema`).
  - `tests/common/`: Factories, mocks, test helpers.
- **Frontend (`SourceCode/front-end/logsmart/`)**: 
  - Svelte 5: Use runes (`$state`, `$derived`, `$effect`) over Svelte 4 stores.
  - `src/lib/api.ts` uses `openapi-fetch`. *Do not manually write API clients.*

## Critical Workflows

### API Spec Generation (Codegen)
Whenever modifying DTOs (`back-end/src/dto.rs`) or Axum routes, regenerate the OpenAPI spec and frontend types in order:
```bash
cd SourceCode/back-end
cargo run --bin gen_spec > ../front-end/logsmart/openapi.json
cd ../front-end/logsmart
bun run gen:api
```

### Adding API Endpoints
1. Add DTO in `back-end/src/dto.rs`.
2. Implement handler in `handlers/` and logic in `services/`.
3. Add DB queries to `db.rs` or `logs_db.rs`.
4. Register route in `main.rs`.
5. Run the Codegen commands above.

### Adding Template Fields
1. **Backend**: Add to `TemplateField` in `logs_db.rs` (no MongoDB migration needed).
2. **Frontend**: Add component in `template-designer/`.

### Database & SQLx Quirks
- **TIMESTAMPTZ**: Always use `TIMESTAMPTZ` in Postgres, never `TIMESTAMP`.
- **SQLx Offline Mode**: SQLx query checks require an active DB or offline cache. When modifying queries, update `.sqlx/` cache (via `cargo sqlx prepare`) or rely on `SQLX_OFFLINE=true` for builds/tests without a live DB.

## Testing & Verification

### Backend (Rust)
```bash
cd SourceCode/back-end
cargo test                                       # All tests
cargo test --test auth_tests                     # Specific integration test file
cargo test --test service_tests log_entry_service # Specific module
cargo clippy --fix --allow-dirty --allow-staged  # Auto-fix linting
```

### Frontend (SvelteKit)
```bash
cd SourceCode/front-end/logsmart
bun run format       # Prettier formatting
bun run check        # Type check (TypeScript + Svelte)
bun run lint         # Prettier + ESLint
bun x playwright test # E2E tests
```

## Conventions
- **Commits**: Prefix with `Back-End:`, `Front-End:`, `Fix:`, `Docs:`, or `Chore:`.
- **Errors**: Return meaningful messages; never expose DB/internal errors to clients.