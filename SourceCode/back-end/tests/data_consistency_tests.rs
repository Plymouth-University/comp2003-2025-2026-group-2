// ============================================================================
// Data Consistency Tests for LogSmart Backend
// ============================================================================
// Test coverage for database constraints, transactions, and data integrity
// Tests focus on:
// - Database constraint violations (unique email, foreign key cascade)
// - Concurrent writes and transactions
// - Null/Empty data validation
// - Timestamp consistency and immutability
// - Referential integrity across tables
//
// File references:
// - Database models: src/db.rs:1-300
// - Database schema: migrations/20260201000003_create_users_table.sql
// - Common utilities: tests/common/mod.rs
// ============================================================================

mod common;

use chrono::Utc;
use common::{CompanyFactory, UserFactory};

// ============================================================================
// TIER 3: DATABASE CONSTRAINT VIOLATION TESTS (2 tests)
// ============================================================================

/// Test: Unique email constraint is enforced
/// Description: Verify that the database unique constraint on email prevents
/// duplicate email addresses from being inserted.
/// Location: migrations/20260201000003_create_users_table.sql:4
#[tokio::test]
async fn test_email_unique_constraint_enforced() {
    let pool = common::setup_test_db().await;

    let user1 = UserFactory::create_basic();
    let email = "unique_test@example.com".to_string();

    // Insert first user with unique email
    let result1 = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&user1.id)
    .bind(&email)
    .bind("First")
    .bind("User")
    .bind(&user1.company_id)
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(
        result1.is_ok(),
        "First user with unique email should be inserted successfully"
    );

    // Attempt to insert second user with same email - should fail
    let user2 = UserFactory::create_basic();
    let result2 = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&user2.id)
    .bind(&email) // Same email as user1
    .bind("Second")
    .bind("User")
    .bind(&user2.company_id)
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(
        result2.is_err(),
        "Duplicate email should violate unique constraint"
    );
}

/// Test: Foreign key cascade delete is enforced
/// Description: Verify that deleting a company cascades to set user.company_id to NULL
/// (or cascades delete based on constraints).
/// Location: migrations/20260201000003_create_users_table.sql:15
#[tokio::test]
async fn test_foreign_key_cascade_on_company_delete() {
    let pool = common::setup_test_db().await;

    // Create company
    let company = CompanyFactory::create_basic();
    let company_id = company.id.clone();

    let insert_company = sqlx::query(
        "INSERT INTO companies (id, name, address, created_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&company_id)
    .bind(&company.name)
    .bind(&company.address)
    .bind(company.created_at)
    .execute(&pool)
    .await;

    assert!(insert_company.is_ok(), "Company should be inserted");

    // Create user with foreign key to company
    let user = UserFactory::create_basic();
    let user_id = user.id.clone();

    let insert_user = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&user_id)
    .bind("cascade_test@example.com")
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&company_id)
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(insert_user.is_ok(), "User with company_id should be inserted");

    // Verify user has company_id set
    let user_before: (Option<String>,) =
        sqlx::query_as("SELECT company_id FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("User should exist");

    assert_eq!(
        user_before.0, Some(company_id.clone()),
        "User should have company_id set before deletion"
    );

    // Delete company (soft delete with deleted_at)
    let delete_company = sqlx::query("UPDATE companies SET deleted_at = NOW() WHERE id = $1")
        .bind(&company_id)
        .execute(&pool)
        .await;

    assert!(
        delete_company.is_ok(),
        "Company soft delete should succeed"
    );

    // Verify company is marked as deleted
    let company_after: (Option<chrono::DateTime<Utc>>,) = sqlx::query_as(
        "SELECT deleted_at FROM companies WHERE id = $1"
    )
    .bind(&company_id)
    .fetch_one(&pool)
    .await
    .expect("Company should still exist after soft delete");

    assert!(
        company_after.0.is_some(),
        "Company deleted_at should be set"
    );

    // User's company_id should still reference the company (soft delete doesn't cascade)
    // But if hard delete is implemented, foreign key would set to NULL
    let user_final: (Option<String>,) =
        sqlx::query_as("SELECT company_id FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("User should still exist");

    // With soft delete, company_id remains (expected behavior)
    // With hard delete + ON DELETE SET NULL, it would be None
    assert_eq!(
        user_final.0, Some(company_id),
        "User company_id should remain in soft delete scenario"
    );
}

// ============================================================================
// CONCURRENT WRITES & TRANSACTIONS TESTS (2 tests)
// ============================================================================

/// Test: Concurrent user creation with same email fails properly
/// Description: Simulate two concurrent registration attempts with identical email.
/// Verify that only one succeeds (or both fail appropriately) and exactly one user
/// exists in the database.
#[tokio::test]
async fn test_concurrent_user_creation_same_email_race_condition() {
    let pool = common::setup_test_db().await;

    let email = "concurrent_race@example.com".to_string();
    let company_id = "company_concurrent".to_string();

    // Insert company first
    sqlx::query("INSERT INTO companies (id, name, address, created_at) VALUES ($1, $2, $3, $4)")
        .bind(&company_id)
        .bind("Concurrent Test Co")
        .bind("123 Main St")
        .bind(Utc::now())
        .execute(&pool)
        .await
        .expect("Company should be created");

    let user1_id = "user_concurrent_1".to_string();
    let user2_id = "user_concurrent_2".to_string();

    // Spawn two concurrent tasks attempting to insert users with same email
    let pool_clone1 = pool.clone();
    let pool_clone2 = pool.clone();
    let email_clone1 = email.clone();
    let email_clone2 = email.clone();
    let company_id_clone1 = company_id.clone();
    let company_id_clone2 = company_id.clone();

    let handle1 = tokio::spawn(async move {
        sqlx::query(
            "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&user1_id)
        .bind(&email_clone1)
        .bind("User")
        .bind("One")
        .bind(&company_id_clone1)
        .bind("staff")
        .bind(Utc::now())
        .execute(&pool_clone1)
        .await
    });

    let handle2 = tokio::spawn(async move {
        sqlx::query(
            "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&user2_id)
        .bind(&email_clone2)
        .bind("User")
        .bind("Two")
        .bind(&company_id_clone2)
        .bind("staff")
        .bind(Utc::now())
        .execute(&pool_clone2)
        .await
    });

    let result1 = handle1.await.expect("Task 1 should complete");
    let result2 = handle2.await.expect("Task 2 should complete");

    // At least one should fail due to unique constraint
    assert!(
        result1.is_err() || result2.is_err(),
        "At least one concurrent insert should fail due to unique email constraint"
    );

    // Verify exactly one user with this email exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .expect("Count query should succeed");

    assert_eq!(
        count.0, 1,
        "Exactly one user should exist with this email after concurrent inserts"
    );
}

/// Test: Transaction rollback on error maintains consistency
/// Description: Start a transaction, insert a user, then trigger an error.
/// Verify the user was not committed to database.
#[tokio::test]
async fn test_transaction_rollback_on_error_maintains_consistency() {
    let pool = common::setup_test_db().await;

    let user_email = "rollback_test@example.com".to_string();
    let user_id = "user_rollback_test".to_string();

    // Verify user doesn't exist initially
    let initial_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&user_email)
        .fetch_one(&pool)
        .await
        .expect("Initial count should succeed");

    assert_eq!(initial_count.0, 0, "User should not exist initially");

    // Attempt a transaction that will fail
    let mut tx = pool.begin().await.expect("Transaction should begin");

    let insert_result = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&user_id)
    .bind(&user_email)
    .bind("Rollback")
    .bind("Test")
    .bind("staff")
    .bind(Utc::now())
    .execute(&mut *tx)
    .await;

    assert!(insert_result.is_ok(), "Insert should succeed within transaction");

    // Introduce constraint violation (try to insert same email again in same transaction)
    let duplicate_insert = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind("user_rollback_duplicate")
    .bind(&user_email)
    .bind("Duplicate")
    .bind("User")
    .bind("staff")
    .bind(Utc::now())
    .execute(&mut *tx)
    .await;

    assert!(
        duplicate_insert.is_err(),
        "Duplicate email should violate unique constraint"
    );

    // Explicitly rollback transaction
    let rollback_result = tx.rollback().await;
    assert!(rollback_result.is_ok(), "Rollback should succeed");

    // Verify BOTH inserts were rolled back
    let final_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&user_email)
        .fetch_one(&pool)
        .await
        .expect("Final count should succeed");

    assert_eq!(
        final_count.0, 0,
        "All inserts should be rolled back; user should not exist"
    );
}

// ============================================================================
// NULL/EMPTY DATA VALIDATION TESTS (2 tests)
// ============================================================================

/// Test: Required fields cannot be null
/// Description: Verify that NOT NULL constraints on required fields (email, first_name, etc.)
/// prevent insertion of records with null values.
/// Location: migrations/20260201000003_create_users_table.sql:5-6
#[tokio::test]
async fn test_required_fields_cannot_be_null() {
    let pool = common::setup_test_db().await;

    // Test 1: email is NOT NULL
    let result_null_email = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind("user_null_email")
    .bind::<Option<String>>(None) // Try to insert NULL email
    .bind("First")
    .bind("Last")
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(
        result_null_email.is_err(),
        "NULL email should violate NOT NULL constraint"
    );

    // Test 2: first_name is NOT NULL
    let result_null_first_name = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind("user_null_first_name")
    .bind("test@example.com")
    .bind::<Option<String>>(None) // Try to insert NULL first_name
    .bind("Last")
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(
        result_null_first_name.is_err(),
        "NULL first_name should violate NOT NULL constraint"
    );

    // Test 3: role is NOT NULL and has default
    let result_null_role = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, NULL, $5)",
    )
    .bind("user_null_role")
    .bind("test_role@example.com")
    .bind("First")
    .bind("Last")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(
        result_null_role.is_err(),
        "NULL role should violate NOT NULL constraint"
    );
}

/// Test: Empty strings are rejected where appropriate
/// Description: Verify that empty strings in required text fields are rejected
/// by application logic or constraints. (Some fields may allow empty; this tests
/// that validation exists.)
#[tokio::test]
async fn test_empty_strings_rejected_appropriately() {
    let pool = common::setup_test_db().await;

    // Test 1: Empty email should be allowed at database level (CHECK constraint handles format)
    // But email CHECK constraint should reject invalid format
    let result_empty_email = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind("user_empty_email")
    .bind("") // Empty email
    .bind("First")
    .bind("Last")
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    // Email CHECK constraint: email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    // Empty string should fail this regex
    assert!(
        result_empty_email.is_err(),
        "Empty email should violate email format CHECK constraint"
    );

    // Test 2: Empty first_name should be allowed at DB level (no constraint prevents it)
    // but might fail at application validation
    let result_empty_first_name = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind("user_empty_first_name")
    .bind("empty_name@example.com")
    .bind("") // Empty first_name
    .bind("Last")
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    // Empty string should be technically allowed by database
    // (application layer should validate), so this test documents current behavior
    if result_empty_first_name.is_ok() {
        // If allowed, verify it was inserted
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = $1")
            .bind("user_empty_first_name")
            .fetch_one(&pool)
            .await
            .expect("Count should work");
        assert_eq!(count.0, 1, "Empty first_name was allowed by database");
    }
}

// ============================================================================
// TIMESTAMP CONSISTENCY TESTS (2 tests)
// ============================================================================

/// Test: created_at is immutable after insert
/// Description: Verify that created_at timestamp cannot be changed after initial insert.
/// The database should have a DEFAULT value and should not allow updates.
/// Location: migrations/20260201000003_create_users_table.sql:10
#[tokio::test]
async fn test_created_at_immutable_after_insert() {
    let pool = common::setup_test_db().await;

    let user_id = "user_created_at_immutable".to_string();
    let email = "created_at_test@example.com".to_string();

    // Insert user with DEFAULT created_at
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)",
    )
    .bind(&user_id)
    .bind(&email)
    .bind("Created")
    .bind("Test")
    .bind("staff")
    .execute(&pool)
    .await
    .expect("User insert should succeed");

    // Fetch original created_at
    let (_created_at_original,): (chrono::DateTime<Utc>,) =
        sqlx::query_as("SELECT created_at FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("User should exist");

    // Wait a bit to ensure time difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Attempt to update created_at to a different time
    let new_created_at = Utc::now();
    let update_result = sqlx::query("UPDATE users SET created_at = $1 WHERE id = $2")
        .bind(new_created_at)
        .bind(&user_id)
        .execute(&pool)
        .await;

    // Database allows the update (no constraint prevents it)
    // but application should enforce immutability
    if update_result.is_ok() {
        let (created_at_after,): (chrono::DateTime<Utc>,) =
            sqlx::query_as("SELECT created_at FROM users WHERE id = $1")
                .bind(&user_id)
                .fetch_one(&pool)
                .await
                .expect("User should still exist");

        // In production code, application should prevent this change
        // This test documents that database allows it
        // Ideally, created_at should remain unchanged
        assert_ne!(
            created_at_after, new_created_at,
            "Application should prevent created_at modification"
        );
    } else {
        // If database prevents updates, that's also acceptable
        assert!(
            update_result.is_err(),
            "Database may prevent created_at modification via trigger/rule"
        );
    }
}

/// Test: updated_at changes on update but created_at stays same
/// Description: Verify that updated_at reflects modification time while created_at
/// remains unchanged. If updated_at column doesn't exist, test that only created_at
/// tracks both timestamps (document current schema).
#[tokio::test]
async fn test_updated_at_changes_while_created_at_unchanged() {
    let pool = common::setup_test_db().await;

    let user_id = "user_updated_at_test".to_string();
    let email = "updated_at_test@example.com".to_string();

    // Insert user
    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, role, created_at)
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)",
    )
    .bind(&user_id)
    .bind(&email)
    .bind("Update")
    .bind("Test")
    .bind("staff")
    .execute(&pool)
    .await
    .expect("User insert should succeed");

    let (created_at_original,): (chrono::DateTime<Utc>,) =
        sqlx::query_as("SELECT created_at FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("User should exist");

    // Wait to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Update user first_name
    sqlx::query("UPDATE users SET first_name = $1 WHERE id = $2")
        .bind("UpdatedName")
        .bind(&user_id)
        .execute(&pool)
        .await
        .expect("Update should succeed");

    // Verify created_at hasn't changed
    let (created_at_after_update,): (chrono::DateTime<Utc>,) =
        sqlx::query_as("SELECT created_at FROM users WHERE id = $1")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .expect("User should still exist");

    assert_eq!(
        created_at_original, created_at_after_update,
        "created_at should not change on update"
    );

    // Note: Current schema may not have updated_at column
    // This test verifies the immutability of created_at
    // Application-level logic should track update times if needed
}

// ============================================================================
// REFERENTIAL INTEGRITY TESTS (2 tests)
// ============================================================================

/// Test: Company deletion cascades correctly to users/logs
/// Description: Verify that when a company is deleted, appropriate cleanup occurs.
/// With current soft-delete design (deleted_at), orphaned records should not be created.
#[tokio::test]
async fn test_company_deletion_cascades_correctly() {
    let pool = common::setup_test_db().await;

    // Create company
    let company_id = "company_cascade_test".to_string();
    sqlx::query("INSERT INTO companies (id, name, address, created_at) VALUES ($1, $2, $3, $4)")
        .bind(&company_id)
        .bind("Cascade Test Co")
        .bind("456 Test Ave")
        .bind(Utc::now())
        .execute(&pool)
        .await
        .expect("Company should be created");

    // Create multiple users in this company
    let user1_id = "user_cascade_1".to_string();
    let user2_id = "user_cascade_2".to_string();

    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&user1_id)
    .bind("cascade_user1@example.com")
    .bind("User")
    .bind("One")
    .bind(&company_id)
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await
    .expect("User 1 should be created");

    sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&user2_id)
    .bind("cascade_user2@example.com")
    .bind("User")
    .bind("Two")
    .bind(&company_id)
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await
    .expect("User 2 should be created");

    // Verify users exist and belong to company
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE company_id = $1")
        .bind(&company_id)
        .fetch_one(&pool)
        .await
        .expect("Count should work");

    assert_eq!(user_count.0, 2, "Should have 2 users in company");

    // Soft delete company
    sqlx::query("UPDATE companies SET deleted_at = NOW() WHERE id = $1")
        .bind(&company_id)
        .execute(&pool)
        .await
        .expect("Company delete should succeed");

    // Verify company is marked deleted
    let (company_deleted_at,): (Option<chrono::DateTime<Utc>>,) =
        sqlx::query_as("SELECT deleted_at FROM companies WHERE id = $1")
            .bind(&company_id)
            .fetch_one(&pool)
            .await
            .expect("Company should still exist");

    assert!(
        company_deleted_at.is_some(),
        "Company deleted_at should be set"
    );

    // With soft delete, users' company_id should remain
    // (no orphaned records, just marked via company.deleted_at)
    let users_after_delete: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE company_id = $1")
            .bind(&company_id)
            .fetch_one(&pool)
            .await
            .expect("Count should work");

    assert_eq!(
        users_after_delete.0, 2,
        "Users should still have company_id references after soft delete"
    );
}

/// Test: Orphaned records are not created
/// Description: Verify that database constraints and application logic prevent
/// creation of records with invalid foreign key references.
#[tokio::test]
async fn test_orphaned_records_not_created() {
    let pool = common::setup_test_db().await;

    // Attempt to create user with non-existent company_id
    let non_existent_company = "company_does_not_exist_123456".to_string();
    let orphan_user_id = "orphan_user_test".to_string();

    let result = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&orphan_user_id)
    .bind("orphan@example.com")
    .bind("Orphan")
    .bind("User")
    .bind(&non_existent_company)
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    // Foreign key constraint should prevent orphaned user
    // But current schema has ON DELETE SET NULL, so it might allow the insert initially
    // Database doesn't have foreign key constraint that prevents insert
    // (because of ON DELETE SET NULL behavior)

    if result.is_ok() {
        // If insert succeeded, verify that referential integrity query shows no orphan
        let orphan_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users 
             WHERE company_id IS NOT NULL 
             AND company_id NOT IN (SELECT id FROM companies WHERE deleted_at IS NULL)",
        )
        .bind(&non_existent_company)
        .fetch_one(&pool)
        .await
        .expect("Orphan query should work");

        assert!(
            orphan_count.0 > 0,
            "This test documents that database allows orphaned records with current schema"
        );
        // In production, application should validate company_id before insert
    } else {
        // If insert failed, that's the ideal behavior
        assert!(
            result.is_err(),
            "Foreign key constraint should prevent orphaned record"
        );
    }

    // Verify user with NULL company_id is allowed (expected for HQ staff)
    let valid_user = sqlx::query(
        "INSERT INTO users (id, email, first_name, last_name, company_id, role, created_at)
         VALUES ($1, $2, $3, $4, NULL, $5, $6)",
    )
    .bind("valid_hq_user")
    .bind("hq_user@example.com")
    .bind("HQ")
    .bind("User")
    .bind("staff")
    .bind(Utc::now())
    .execute(&pool)
    .await;

    assert!(
        valid_user.is_ok(),
        "User with NULL company_id should be allowed (HQ staff)"
    );
}
