# LogSmart - Agent Guidelines

## Quick Info

- **Stack**: Rust 2024/Axum backend (port 6767), SvelteKit 5 frontend (port 5173), PostgreSQL + MongoDB
- **Backend**: `SourceCode/back-end/` | **Frontend**: `SourceCode/front-end/logsmart/`
- **Full docs**: `SourceCode/docs/PROJECT.md`
- **Nix**: Run `direnv exec . <command>` in `back-end/` for Rust toolchain + required env vars

---

## Commands

### Backend (Rust)

```bash
cd SourceCode/back-end

# Run all tests
cargo test

# Run single test by name
cargo test test_name_here

# Run single test file
cargo test --test auth_tests

# Run specific test module in tests/services/
cargo test --test service_tests log_entry_service

# Format + lint
cargo fmt && cargo clippy

# Lint with auto-fix
cargo clippy --fix --allow-dirty --allow-staged

# Start databases (docker)
docker compose up -d
```

### Frontend (Svelte/Bun)

```bash
cd SourceCode/front-end/logsmart

bun run dev          # Dev server (port 5173)
bun run build        # Production build
bun run format       # Prettier (tabs, single quotes, 100 char width)
bun run lint         # Prettier check + ESLint
bun run check        # TypeScript + Svelte type check

bun x playwright test                    # All e2e tests
bun x playwright test tests/01-register_company.spec.ts  # Single test file
```

### API Spec Generation

```bash
cd SourceCode/back-end
cargo run --bin gen_spec > ../front-end/logsmart/openapi.json
cd ../front-end/logsmart && bun run gen:api
```

---

## Code Style

### Rust

- **Formatter**: `cargo fmt` | **Linter**: `cargo clippy` before every commit
- **Error handling**: `anyhow` crate (`Result<T>` alias, `.context()` for wrapping)
- **Async**: `async/await` with Tokio; `#[async_trait]` for trait methods
- **Naming**: `snake_case` functions/vars, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants
- **Imports**: Group with blank lines — std → external crates → `crate::` modules
- **DTOs**: Derive `Serialize, Deserialize` + utoipa `ToSchema` for OpenAPI

### Svelte/TypeScript

- **Formatter**: Prettier — tabs, single quotes, no trailing commas, 100 char print width
- **Linting**: ESLint + `eslint-plugin-svelte` + `typescript-eslint`
- **TypeScript**: Always explicit types for function params and return types
- **Svelte 5**: Use runes (`$state`, `$derived`, `$effect`) over stores for new code
- **CSS**: Tailwind v4 (see `src/app.css`)

### General

- **Imports order**: built-in → external → relative paths
- **Errors**: Return meaningful messages; never expose internals to clients
- **Commits**: Prefix with `Back-End:`, `Front-End:`, `Fix:`, `Docs:`, `Chore:`

---

## Project Structure

```
SourceCode/
├── back-end/
│   ├── src/
│   │   ├── main.rs           # Entry point, Axum router
│   │   ├── lib.rs            # Module exports
│   │   ├── db.rs             # PostgreSQL (SQLx) models/queries
│   │   ├── logs_db.rs        # MongoDB models/queries
│   │   ├── dto.rs            # Request/Response DTOs
│   │   ├── handlers/         # HTTP handlers
│   │   └── services/         # Business logic
│   ├── tests/
│   │   ├── common/           # Test helpers (factories, mocks, config)
│   │   ├── services/         # Per-service test files
│   │   └── *.rs              # Integration test files
│   ├── migrations/           # SQLx migrations
│   └── .sqlx/                # Offline query cache (SQLX_OFFLINE=true)
└── front-end/logsmart/
    ├── src/routes/           # SvelteKit pages
    ├── src/lib/
    │   ├── api.ts            # Generated API client (openapi-fetch)
    │   ├── api-types.d.ts    # Generated types
    │   └── components/       # Reusable Svelte components
    └── tests/                # Playwright e2e tests
```

---

## Key Patterns

### Adding API Endpoint
1. DTO in `back-end/src/dto.rs` → handler in `handlers/` → service in `services/` → DB in `db.rs`
2. Register route in `main.rs`
3. Regenerate spec: `cargo run --bin gen_spec > .../openapi.json` then `bun run gen:api`

### Adding Template Field
1. Backend: Add to `TemplateField` in `logs_db.rs` (no MongoDB migration needed)
2. Frontend: Add component in `template-designer/`

---

## Git Conventions

- **Branches**: `feature/`, `fix/`, `refactor/`, `test/`, `chore/`
- **Commits**: `<Area>: <Subject>` in imperative mood. E.g. `Back-End: Add rate limiting middleware`
- **Never commit secrets**

---

## Testing

- **Backend unit tests**: Inline with `#[cfg(test)]` modules in `src/`
- **Backend integration tests**: `back-end/tests/` — use helpers in `tests/common/` (factories, mocks)
- **Property tests**: Use `proptest` crate (see `tests/property_tests.rs`)
- **Frontend e2e**: `front-end/logsmart/tests/` (Playwright)
- **SQLx offline mode**: Set `SQLX_OFFLINE=true` when running tests/builds without a live DB

## Database

- Always use `TIMESTAMPTZ` over `TIMESTAMP`
- SQLx uses compile-time checked queries (`.sqlx/` cache for offline mode)

## Code Review

- When given a code review result, fix the issue and create a git commit