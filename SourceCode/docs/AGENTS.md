# LogSmart - Agent Guidelines

## Quick Info

- **Stack**: Rust/Axum backend (port 6767), SvelteKit frontend (port 5173), PostgreSQL + MongoDB
- **Backend**: `SourceCode/back-end/`
- **Frontend**: `SourceCode/front-end/logsmart/`
- **Full docs**: See `SourceCode/docs/PROJECT.md`

---

## Commands

### Backend (Rust)

```bash
cd SourceCode/back-end

# Run all tests
cargo test

# Run single test (by name)
cargo test test_name_here

# Run tests in specific file
cargo test --test auth_tests

# Format code
cargo fmt

# Lint with auto-fix
cargo clippy --fix --allow-dirty --allow-staged

# Lint only
cargo clippy

# Run server (requires docker)
docker compose up -d
```

### Frontend (Svelte/Bun)

```bash
cd SourceCode/front-end/logsmart

# Dev server
bun run dev

# Build
bun run build

# Format
bun run format

# Lint
bun run lint

# Type check
bun run check

# Run Playwright tests
bun x playwright test

# Run single test file
bun x playwright test tests/01-register_company.spec.ts
```

### API Spec Generation

```bash
cd SourceCode/back-end
cargo run --bin gen_spec > ../front-end/logsmart/openapi.json

cd SourceCode/front-end/logsmart
bun run gen:api
```

---

## Code Style

### Rust

- **Formatting**: Use `cargo fmt` (Rust's standard formatter)
- **Linting**: Run `cargo clippy` before committing
- **Error handling**: Use `anyhow` crate for application errors
- **Async**: Use `async/await` with Tokio runtime
- **Naming**: snake_case for variables/functions, PascalCase for types
- **Imports**: Group: std → external (crates.io) → internal (project modules)

```rust
use std::collections::HashMap;

use anyhow::{Context, Result};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::db::UserRecord;
```

### Svelte/TypeScript

- **Formatting**: Use `bun run format` (Prettier)
- **Linting**: Use `bun run lint` (ESLint + Prettier)
- **TypeScript**: Always use explicit types for function params/returns
- **Svelte 5**: Use runes (`$state`, `$derived`, `$effect`) instead of stores where possible

```typescript
import type { SvelteComponent } from 'svelte';
import { derived } from 'svelte/store';

// Use runes for new components
let count = $state(0);
let doubled = $derived(count * 2);
```

### General

- **Naming**:
  - Rust: `snake_case` functions, `PascalCase` types
  - TS/JS: `camelCase` variables, `PascalCase` components/types
- **Imports**: Order: built-in → external → relative
- **Errors**: Return meaningful error messages; never expose internals
- **Commits**: Prefix with `Back-End:`, `Front-End:`, `Fix:`, `Docs:`, etc.

---

## Project Structure

```
SourceCode/
├── back-end/
│   ├── src/
│   │   ├── main.rs           # Entry point, router setup
│   │   ├── lib.rs            # Module exports
│   │   ├── db.rs             # PostgreSQL models/queries
│   │   ├── logs_db.rs        # MongoDB models/queries
│   │   ├── dto.rs            # Request/Response DTOs
│   │   ├── handlers/         # HTTP handlers
│   │   └── services/         # Business logic
│   ├── migrations/           # SQLx migrations
│   └── tests/                # Integration tests
└── front-end/logsmart/
    ├── src/
    │   ├── routes/           # SvelteKit pages
    │   ├── lib/
    │   │   ├── api.ts        # API client
    │   │   ├── components/  # Reusable components
    │   │   └── stores/       # Svelte stores
    │   └── app.css
    └── tests/                # Playwright e2e tests
```

---

## Key Patterns

### Adding API Endpoint

1. Define DTO in `back-end/src/dto.rs`
2. Add handler in `back-end/src/handlers/*.rs`
3. Add service in `back-end/src/services/*.rs`
4. Add DB query in `back-end/src/db.rs`
5. Register route in `back-end/src/main.rs`
6. Regenerate API: `cargo run --bin gen_spec > ...`
7. Regenerate types: `bun run gen:api`

### Adding Template Field

1. Backend: Add to `TemplateField` in `logs_db.rs`
2. Frontend: Add component in `template-designer/`
3. No migration needed (MongoDB)

---

## Git Conventions

- **Branches**: `feature/`, `fix/`, `refactor/`, `test/`, `chore/`
- **Commits**: `<Area>: <Subject>` (imperative mood)
  - Example: `Front-End: Add user invitation modal`
- **Never commit secrets** (env vars, credentials)

---

## Testing

- Backend unit tests: Inline in `src/` with `#[cfg(test)]`
- Backend integration tests: `back-end/tests/`
- Frontend e2e: `front-end/logsmart/tests/` (Playwright)

## Database
- Always use TIMESTAMPTZ over TIMESTAMP
