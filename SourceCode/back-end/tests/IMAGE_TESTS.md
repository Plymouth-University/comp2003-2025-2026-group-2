# Image Upload Tests

This document describes the tests created for the image uploader feature.

## Unit Tests

### 1. Image Handlers Unit Tests (`src/handlers/image_handlers.rs`)

Tests for ObjectId parsing and validation:
- `test_object_id_parsing_valid` - Tests valid ObjectId strings (24 hex characters)
- `test_object_id_parsing_invalid` - Tests invalid ObjectId strings
- `test_object_id_parsing_empty` - Tests empty string rejection

### 2. GridFS Unit Tests (`src/logs_db.rs`)

Tests for MongoDB GridFS functionality:
- `test_object_id_generation` - Verifies unique ID generation and format
- `test_object_id_parsing` - Tests round-trip parsing of ObjectIds
- `test_object_id_parsing_invalid` - Tests rejection of malformed IDs
- `test_object_id_timestamp` - Verifies timestamps are reasonable

## Integration Tests

### 3. Image Integration Tests (`tests/image_integration_tests.rs`)

E2E tests for the image upload endpoints:

- `test_image_upload_request_validation` - Tests authentication requirement
- `test_image_upload_with_valid_token` - Tests successful upload flow
- `test_image_upload_file_size_validation` - Tests 5MB file size limit
- `test_image_get_invalid_id` - Tests error handling for invalid image IDs
- `test_image_delete_invalid_id` - Tests delete endpoint with invalid IDs
- `test_user_without_company_cannot_upload` - Tests company requirement

## Running the Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only image-related tests
cargo test image
cargo test gridfs

# Run integration tests (requires server running)
cargo test --test image_integration_tests

# Run with output
cargo test -- --nocapture
```

## Test Requirements

- **Unit tests**: No external dependencies, run quickly
- **Integration tests**: Require:
  - PostgreSQL test database (default: `postgres://admin:adminpassword@localhost:5432/logsmartdb`)
  - Running backend server on `http://localhost:3000`
  - Set `TEST_DATABASE_URL` environment variable to override

## Test Coverage

The tests cover:
- ✅ ObjectId parsing and validation
- ✅ Authentication requirements
- ✅ Company membership requirements
- ✅ File size validation (5MB limit)
- ✅ Error handling for invalid inputs
- ✅ API endpoint structure and responses
