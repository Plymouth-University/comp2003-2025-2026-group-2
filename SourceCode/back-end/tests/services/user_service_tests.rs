use crate::services::user_service::UserService;
use axum::http::StatusCode;
use back_end::db::{self, UserRole};
use back_end::tests::common::{factories::*, setup_test_db};
use serde_json::json;

#[tokio::test]
async fn test_get_user_by_email_success() {
    let pool = setup_test_db().await;
    
    // Create test user
    let user = create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Test successful retrieval
    let result = UserService::get_user_by_email(&pool, "test@example.com").await;
    
    assert!(result.is_ok());
    let retrieved_user = result.unwrap();
    assert_eq!(retrieved_user.email, user.email);
    assert_eq!(retrieved_user.id, user.id);
}

#[tokio::test]
async fn test_get_user_by_email_not_found() {
    let pool = setup_test_db().await;
    
    // Test with non-existent email
    let result = UserService::get_user_by_email(&pool, "nonexistent@example.com").await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_response, json!({"error": "User not found"}));
}

#[tokio::test]
async fn test_get_user_by_id_success() {
    let pool = setup_test_db().await;
    
    // Create test user
    let user = create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Test successful retrieval
    let result = UserService::get_user_by_id(&pool, &user.id).await;
    
    assert!(result.is_ok());
    let retrieved_user = result.unwrap();
    assert_eq!(retrieved_user.id, user.id);
    assert_eq!(retrieved_user.email, user.email);
}

#[tokio::test]
async fn test_get_user_by_id_not_found() {
    let pool = setup_test_db().await;
    
    // Test with non-existent user ID
    let result = UserService::get_user_by_id(&pool, "nonexistent-id").await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_response, json!({"error": "User not found"}));
}

#[tokio::test]
async fn test_update_profile_success() {
    let pool = setup_test_db().await;
    
    // Create test user
    let user = create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Test profile update
    let result = UserService::update_profile(
        &pool,
        &user.id,
        "UpdatedFirstName".to_string(),
        "UpdatedLastName".to_string(),
    ).await;
    
    assert!(result.is_ok());
    let updated_user = result.unwrap();
    assert_eq!(updated_user.first_name, "UpdatedFirstName");
    assert_eq!(updated_user.last_name, "UpdatedLastName");
}

#[tokio::test]
async fn test_get_company_members_success() {
    let pool = setup_test_db().await;
    
    // Create test company and users
    let company_id = "company123".to_string();
    let admin = create_test_user(&pool, "admin@example.com", Some(&company_id)).await;
    let member1 = create_test_user(&pool, "member1@example.com", Some(&company_id)).await;
    let member2 = create_test_user(&pool, "member2@example.com", Some(&company_id)).await;
    
    // Test getting company members
    let result = UserService::get_company_members(&pool, &company_id).await;
    
    assert!(result.is_ok());
    let members = result.unwrap();
    assert_eq!(members.len(), 3);
    
    // Verify all members belong to the company
    for member in members {
        assert_eq!(member.company_id, Some(company_id.clone()));
    }
}

#[tokio::test]
async fn test_get_user_company_id_success() {
    let pool = setup_test_db().await;
    
    // Create test user with company
    let user = create_test_user(&pool, "test@example.com", Some("company123")).await;
    
    // Test getting user company ID
    let result = UserService::get_user_company_id(&pool, &user.id).await;
    
    assert!(result.is_ok());
    let company_id = result.unwrap();
    assert_eq!(company_id, "company123");
}

#[tokio::test]
async fn test_get_user_company_id_no_company() {
    let pool = setup_test_db().await;
    
    // Create test user without company
    let user = create_test_user(&pool, "test@example.com", None).await;
    
    // Test getting company ID for user without company
    let result = UserService::get_user_company_id(&pool, &user.id).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "User is not associated with a company"}));
}

#[tokio::test]
async fn test_admin_update_member_profile_success() {
    let pool = setup_test_db().await;
    
    // Create admin and member users
    let admin = create_test_user_with_role(&pool, "admin@example.com", UserRole::Admin, Some("company123")).await;
    let member = create_test_user_with_role(&pool, "member@example.com", UserRole::Member, Some("company123")).await;
    
    // Test admin updating member profile
    let result = UserService::admin_update_member_profile(
        &pool,
        &admin.id,
        "member@example.com",
        "UpdatedFirstName".to_string(),
        "UpdatedLastName".to_string(),
        UserRole::Admin, // Promote to admin
    ).await;
    
    assert!(result.is_ok());
    let updated_member = result.unwrap();
    assert_eq!(updated_member.first_name, "UpdatedFirstName");
    assert_eq!(updated_member.last_name, "UpdatedLastName");
    assert_eq!(updated_member.role, UserRole::Admin);
}

#[tokio::test]
async fn test_admin_update_member_profile_non_admin_forbidden() {
    let pool = setup_test_db().await;
    
    // Create regular user attempting to act as admin
    let user1 = create_test_user_with_role(&pool, "user1@example.com", UserRole::Member, Some("company123")).await;
    let user2 = create_test_user_with_role(&pool, "user2@example.com", UserRole::Member, Some("company123")).await;
    
    // Test non-admin trying to update member profile
    let result = UserService::admin_update_member_profile(
        &pool,
        &user1.id,
        "user2@example.com",
        "UpdatedFirstName".to_string(),
        "UpdatedLastName".to_string(),
        UserRole::Member,
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Only company admins can update member profiles"}));
}

#[tokio::test]
async fn test_admin_update_member_profile_different_company_forbidden() {
    let pool = setup_test_db().await;
    
    // Create admin and member from different companies
    let admin = create_test_user_with_role(&pool, "admin@example.com", UserRole::Admin, Some("company1")).await;
    let member = create_test_user_with_role(&pool, "member@example.com", UserRole::Member, Some("company2")).await;
    
    // Test admin trying to update member from different company
    let result = UserService::admin_update_member_profile(
        &pool,
        &admin.id,
        "member@example.com",
        "UpdatedFirstName".to_string(),
        "UpdatedLastName".to_string(),
        UserRole::Member,
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Cannot update users from other companies"}));
}

#[tokio::test]
async fn test_admin_delete_member_success() {
    let pool = setup_test_db().await;
    
    // Create admin and member users
    let admin = create_test_user_with_role(&pool, "admin@example.com", UserRole::Admin, Some("company123")).await;
    let member = create_test_user_with_role(&pool, "member@example.com", UserRole::Member, Some("company123")).await;
    
    // Test admin deleting member
    let result = UserService::admin_delete_member(
        &pool,
        &admin.id,
        "member@example.com",
    ).await;
    
    assert!(result.is_ok());
    
    // Verify member is deleted
    let deleted_user = db::get_user_by_email(&pool, "member@example.com").await.unwrap();
    assert!(deleted_user.is_none()); // User should be deleted
}

#[tokio::test]
async fn test_admin_delete_member_self_forbidden() {
    let pool = setup_test_db().await;
    
    // Create admin user
    let admin = create_test_user_with_role(&pool, "admin@example.com", UserRole::Admin, Some("company123")).await;
    
    // Test admin trying to delete themselves
    let result = UserService::admin_delete_member(
        &pool,
        &admin.id,
        "admin@example.com",
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_response, json!({"error": "Cannot delete your own account"}));
}

#[tokio::test]
async fn test_admin_delete_member_non_admin_forbidden() {
    let pool = setup_test_db().await;
    
    // Create regular users
    let user1 = create_test_user_with_role(&pool, "user1@example.com", UserRole::Member, Some("company123")).await;
    let user2 = create_test_user_with_role(&pool, "user2@example.com", UserRole::Member, Some("company123")).await;
    
    // Test non-admin trying to delete member
    let result = UserService::admin_delete_member(
        &pool,
        &user1.id,
        "user2@example.com",
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Only company admins can delete members"}));
}

#[tokio::test]
async fn test_admin_delete_logsmart_admin_forbidden() {
    let pool = setup_test_db().await;
    
    // Create company admin and LogSmart admin
    let admin = create_test_user_with_role(&pool, "admin@example.com", UserRole::Admin, Some("company123")).await;
    let logsmart_admin = create_test_user_with_role(&pool, "logsmart@example.com", UserRole::LogSmartAdmin, Some("company123")).await;
    
    // Test company admin trying to delete LogSmart admin
    let result = UserService::admin_delete_member(
        &pool,
        &admin.id,
        "logsmart@example.com",
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Cannot delete LogSmart internal admin users"}));
}

#[tokio::test]
async fn test_admin_update_logsmart_admin_forbidden() {
    let pool = setup_test_db().await;
    
    // Create company admin and LogSmart admin
    let admin = create_test_user_with_role(&pool, "admin@example.com", UserRole::Admin, Some("company123")).await;
    let logsmart_admin = create_test_user_with_role(&pool, "logsmart@example.com", UserRole::LogSmartAdmin, Some("company123")).await;
    
    // Test company admin trying to update LogSmart admin
    let result = UserService::admin_update_member_profile(
        &pool,
        &admin.id,
        "logsmart@example.com",
        "UpdatedFirstName".to_string(),
        "UpdatedLastName".to_string(),
        UserRole::LogSmartAdmin,
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Cannot modify LogSmart internal admin users"}));
}