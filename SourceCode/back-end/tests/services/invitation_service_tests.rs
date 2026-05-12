use crate::services::invitation_service::InvitationService;
use axum::http::StatusCode;
use back_end::db::{self, UserRole};
use back_end::tests::common::{factories::*, setup_test_db};
use chrono::{Duration, Utc};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_send_invitation_success() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin user
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company123")).await;
    
    // Test successful invitation sending
    let result = InvitationService::send_invitation(
        &pool,
        admin.id.clone(),
        format!("admin{}@example.com", unique_id),
        format!("newuser{}@example.com", unique_id),
        "company123".to_string(),
        Some("127.0.0.1".to_string()),
        Some("test-agent".to_string()),
    ).await;
    
    assert!(result.is_ok());
    let (invitation_id, expires_at) = result.unwrap();
    assert!(!invitation_id.is_empty());
    assert!(expires_at > Utc::now());
}

#[tokio::test]
async fn test_send_invitation_user_already_exists() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin and existing user
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company123")).await;
    let existing_user = create_test_user(&pool, &format!("existing{}@example.com", unique_id), Some("company123")).await;
    
    // Test invitation to existing user
    let result = InvitationService::send_invitation(
        &pool,
        admin.id.clone(),
        format!("admin{}@example.com", unique_id),
        format!("existing{}@example.com", unique_id),
        "company123".to_string(),
        None,
        None,
    ).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_response, json!({"error": "User already registered"}));
}

#[tokio::test]
async fn test_accept_invitation_success() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create invitation
    let invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    
    // Test successful invitation acceptance
    let result = InvitationService::accept_invitation(&pool, &invitation.token).await;
    
    assert!(result.is_ok());
    let (accepted_invitation, expires_at) = result.unwrap();
    assert_eq!(accepted_invitation.id, invitation.id);
    assert_eq!(accepted_invitation.email, invitation.email);
}

#[tokio::test]
async fn test_accept_invitation_not_found() {
    let pool = setup_test_db().await;
    
    // Test acceptance with invalid token
    let result = InvitationService::accept_invitation(&pool, "invalid-token").await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_response, json!({"error": "Invalid or expired invitation"}));
}

#[tokio::test]
async fn test_accept_invitation_expired() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create expired invitation
    let expired_invitation = create_test_invitation_with_expiry(
        &pool,
        "company123",
        &format!("newuser{}@example.com", unique_id),
        Utc::now() - Duration::hours(1) // Expired 1 hour ago
    ).await;
    
    // Test acceptance of expired invitation
    let result = InvitationService::accept_invitation(&pool, &expired_invitation.token).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_response, json!({"error": "Invitation has expired"}));
}

#[tokio::test]
async fn test_get_invitation_details_success() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create company and invitation
    let company = create_test_company(&pool, "Test Company", "123 Test St").await;
    let invitation = create_test_invitation(&pool, &company.id, &format!("newuser{}@example.com", unique_id)).await;
    
    // Test getting invitation details
    let result = InvitationService::get_invitation_details(&pool, &invitation.token).await;
    
    assert!(result.is_ok());
    let (company_name, expires_at) = result.unwrap();
    assert_eq!(company_name, "Test Company");
    assert!(expires_at > Utc::now());
}

#[tokio::test]
async fn test_get_invitation_details_not_found() {
    let pool = setup_test_db().await;
    
    // Test getting details for non-existent invitation
    let result = InvitationService::get_invitation_details(&pool, "invalid-token").await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_response, json!({"error": "Invitation not found"}));
}

#[tokio::test]
async fn test_mark_invitation_accepted_success() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create invitation
    let invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    
    // Test marking invitation as accepted
    let result = InvitationService::mark_invitation_accepted(&pool, &invitation.id).await;
    
    assert!(result.is_ok());
    
    // Verify invitation is marked as accepted
    let updated_invitation = db::get_invitation_by_id(&pool, &invitation.id).await.unwrap().unwrap();
    assert!(updated_invitation.accepted_at.is_some());
}

#[tokio::test]
async fn test_get_pending_invitations_success() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create multiple pending invitations
    let invitation1 = create_test_invitation(&pool, "company123", &format!("user1{}@example.com", unique_id)).await;
    let invitation2 = create_test_invitation(&pool, "company123", &format!("user2{}@example.com", unique_id)).await;
    let invitation3 = create_test_invitation(&pool, "company123", &format!("user3{}@example.com", unique_id)).await;
    
    // Test getting pending invitations
    let result = InvitationService::get_pending_invitations(&pool, "company123").await;
    
    assert!(result.is_ok());
    let pending_invitations = result.unwrap();
    assert_eq!(pending_invitations.len(), 3);
    
    // Verify all invitations belong to the company
    for invitation in pending_invitations {
        assert_eq!(invitation.company_id, "company123");
        assert!(invitation.accepted_at.is_none());
        assert!(invitation.cancelled_at.is_none());
    }
}

#[tokio::test]
async fn test_cancel_invitation_success() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin and invitation
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company123")).await;
    let invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    
    // Test successful invitation cancellation
    let result = InvitationService::cancel_invitation(&pool, &admin.id, &invitation.id).await;
    
    assert!(result.is_ok());
    
    // Verify invitation is cancelled
    let cancelled_invitation = db::get_invitation_by_id(&pool, &invitation.id).await.unwrap().unwrap();
    assert!(cancelled_invitation.cancelled_at.is_some());
}

#[tokio::test]
async fn test_cancel_invitation_non_admin_forbidden() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create regular user and invitation
    let user = create_test_user_with_role(&pool, &format!("user{}@example.com", unique_id), UserRole::Staff, Some("company123")).await;
    let invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    
    // Test non-admin trying to cancel invitation
    let result = InvitationService::cancel_invitation(&pool, &user.id, &invitation.id).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Only company admins can cancel invitations"}));
}

#[tokio::test]
async fn test_cancel_invitation_different_company_forbidden() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin from company1 and invitation for company2
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company1")).await;
    let invitation = create_test_invitation(&pool, "company2", &format!("newuser{}@example.com", unique_id)).await;
    
    // Test admin trying to cancel invitation from different company
    let result = InvitationService::cancel_invitation(&pool, &admin.id, &invitation.id).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_response, json!({"error": "Cannot cancel invitations from other companies"}));
}

#[tokio::test]
async fn test_cancel_invitation_already_accepted() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin and accepted invitation
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company123")).await;
    let mut invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    invitation.accepted_at = Some(Utc::now());
    
    // Update invitation to accepted status
    db::mark_invitation_accepted(&pool, &invitation.id).await.unwrap();
    
    // Test trying to cancel accepted invitation
    let result = InvitationService::cancel_invitation(&pool, &admin.id, &invitation.id).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_response, json!({"error": "Cannot cancel an accepted invitation"}));
}

#[tokio::test]
async fn test_cancel_invitation_already_cancelled() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin and cancelled invitation
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company123")).await;
    let invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    
    // Cancel the invitation first
    db::cancel_invitation(&pool, &invitation.id).await.unwrap();
    
    // Test trying to cancel already cancelled invitation
    let result = InvitationService::cancel_invitation(&pool, &admin.id, &invitation.id).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_response, json!({"error": "Invitation already cancelled"}));
}

#[tokio::test]
async fn test_cancel_invitation_admin_not_found() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create invitation
    let invitation = create_test_invitation(&pool, "company123", &format!("newuser{}@example.com", unique_id)).await;
    
    // Test with non-existent admin ID
    let result = InvitationService::cancel_invitation(&pool, "non-existent-admin", &invitation.id).await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_response, json!({"error": "Admin user not found"}));
}

#[tokio::test]
async fn test_cancel_invitation_not_found() {
    let pool = setup_test_db().await;
    
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    // Create admin
    let admin = create_test_user_with_role(&pool, &format!("admin{}@example.com", unique_id), UserRole::CompanyManager, Some("company123")).await;
    
    // Test with non-existent invitation ID
    let result = InvitationService::cancel_invitation(&pool, &admin.id, "non-existent-invitation").await;
    
    assert!(result.is_err());
    let (status, error_response) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_response, json!({"error": "Invitation not found"}));
}

#[tokio::test]
async fn test_get_pending_invitations_empty() {
    let pool = setup_test_db().await;
    
    // Test getting pending invitations for company with no invitations
    let result = InvitationService::get_pending_invitations(&pool, "company-without-invitations").await;
    
    assert!(result.is_ok());
    let pending_invitations = result.unwrap();
    assert!(pending_invitations.is_empty());
}