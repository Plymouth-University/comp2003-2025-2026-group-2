---
name: logsmart-architect
description: Deeply researches the LogSmart Rust/Svelte stack to maintain agent context.
---

## Core Objectives
1. **Bridge Audit**: Map `dto.rs` structs to the generated `openapi.json` and ensure frontend `api.ts` types are in sync.
2. **Logic Flow**: Trace a request from an Axum route in `main.rs` through `services/` down to `db.rs` or `logs_db.rs`.
3. **Rune Compliance**: Verify that Svelte 5 components in `template-designer/` are using `$state` and `$derived` correctly.

## Commands to Run
- `grep -r "pub struct" SourceCode/back-end/src/dto.rs` (To list data models)
- `grep -r ".route(" SourceCode/back-end/src/main.rs` (To map API entry points)
- `find SourceCode/front-end/logsmart/src -name "*.svelte"` (To index UI components)
