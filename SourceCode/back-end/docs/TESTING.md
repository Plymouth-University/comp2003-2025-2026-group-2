# Testing Documentation

This project uses a comprehensive testing strategy including unit tests, integration tests, property-based tests, concurrency tests, and performance tests.

## Running Tests

### Unit Tests
Run all unit tests:
```bash
cargo test --lib
```

Run tests for specific modules:
```bash
cargo test auth::auth_tests
cargo test db::db_model_tests
cargo test services
```

### Integration Tests
Run integration tests (requires test database environment, currently using SQLite/Postgres):
```bash
cargo test --test api_integration_tests
cargo test --test handlers_tests
cargo test --test db_tests
```

### Advanced Tests

**Property-based Tests:**
Tests email validation and password policy with random inputs.
```bash
cargo test --test property_tests
```

**Concurrency Tests:**
Tests thread safety of shared state components like rate limiting.
```bash
cargo test --test concurrency_tests
```

**Performance Tests:**
Benchmarks critical paths like password hashing.
```bash
cargo test --test performance_tests -- --nocapture
```

## Test Structure

- `src/tests/common/`: Common test utilities, factories, and mocks.
- `src/services/*_tests.rs`: Unit tests for service layer business logic.
- `tests/*.rs`: Integration and advanced test suites.

## Coverage

We aim for high test coverage on core business logic:
- **Auth:** JWT, Password hashing, Policy validation.
- **DB:** Models, Constraints, Relationships.
- **Services:** User management, Invitations, Templates, Logging.
