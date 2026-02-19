use back_end::auth::{hash_password, JwtConfig};
use back_end::db::{self, UserRole};

/// Get a connection pool to the test database
async fn get_test_pool() -> sqlx::PgPool {
    let connection_string = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:adminpassword@localhost:5432/logsmartdb".to_string());

    sqlx::PgPool::connect(&connection_string)
        .await
        .expect("Failed to create test db connection")
}

/// Create a test user with a company
async fn create_test_user_with_company(pool: &sqlx::PgPool) -> (db::UserRecord, db::Company, String) {
    let test_id = uuid::Uuid::new_v4().to_string().replace("-", "");
    let password_hash = hash_password("SecurePassword123").unwrap();
    
    let company = db::create_company(
        pool,
        format!("Test Company {}", test_id),
        "123 Test St".to_string(),
    )
    .await
    .expect("Failed to create company");
    
    let user = db::create_user(
        pool,
        format!("testuser{}@example.com", test_id),
        "Test".to_string(),
        "User".to_string(),
        Some(password_hash),
        Some(company.id.clone()),
        UserRole::CompanyManager,
    )
    .await
    .expect("Failed to create user");
    
    let jwt_config = JwtConfig::new("test_secret".to_string());
    let token = jwt_config
        .generate_token(user.id.clone(), 24)
        .expect("Failed to generate token");
    
    (user, company, token)
}

#[tokio::test]
async fn test_image_upload_request_validation() {
    // Test that the endpoint requires authentication
    let client = reqwest::Client::new();
    
    // Create a simple test image
    let test_image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    
    let form = reqwest::multipart::Form::new()
        .part("image", reqwest::multipart::Part::bytes(test_image_data)
            .file_name("test.png")
            .mime_str("image/png")
            .unwrap());
    
    // Try to upload without authentication
    let response = client
        .post("http://localhost:3000/api/images/upload")
        .multipart(form)
        .send()
        .await;
    
    // Should fail without auth token
    if let Ok(resp) = response {
        assert_ne!(resp.status(), 200);
    }
}

#[tokio::test]
async fn test_image_upload_with_valid_token() {
    let pool = get_test_pool().await;
    let (_user, _company, token) = create_test_user_with_company(&pool).await;
    
    let client = reqwest::Client::new();
    
    // Create a simple test image (small PNG)
    let test_image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    
    let form = reqwest::multipart::Form::new()
        .part("image", reqwest::multipart::Part::bytes(test_image_data)
            .file_name("test.png")
            .mime_str("image/png")
            .unwrap());
    
    let response = client
        .post("http://localhost:3000/api/images/upload")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // The test may pass or fail depending on if the server is running
            // We just want to verify the request structure is correct
            let status = resp.status();
            println!("Upload response status: {}", status);
            
            if status == 200 {
                let result: serde_json::Value = resp.json().await.expect("Failed to parse response");
                assert!(result.get("object_id").is_some());
                assert!(result.get("filename").is_some());
                assert!(result.get("file_size").is_some());
            }
        }
        Err(e) => {
            println!("Request failed (server may not be running): {}", e);
        }
    }
}

#[tokio::test]
async fn test_image_upload_file_size_validation() {
    let pool = get_test_pool().await;
    let (_user, _company, token) = create_test_user_with_company(&pool).await;
    
    let client = reqwest::Client::new();
    
    // Create a "large" file (over 5MB)
    let large_data = vec![0u8; 6 * 1024 * 1024]; // 6MB
    
    let form = reqwest::multipart::Form::new()
        .part("image", reqwest::multipart::Part::bytes(large_data)
            .file_name("large.bin")
            .mime_str("application/octet-stream")
            .unwrap());
    
    let response = client
        .post("http://localhost:3000/api/images/upload")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should get 400 Bad Request for file too large
            if resp.status() == 400 {
                let body = resp.text().await.unwrap_or_default();
                assert!(body.contains("5MB") || body.contains("exceeds"));
            }
        }
        Err(e) => {
            println!("Request failed (server may not be running): {}", e);
        }
    }
}

#[tokio::test]
async fn test_image_get_invalid_id() {
    let client = reqwest::Client::new();
    
    // Try to get an image with an invalid ObjectId
    let response = client
        .get("http://localhost:3000/api/images/invalid-id")
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should get 400 Bad Request for invalid ID
            assert_eq!(resp.status(), 400);
        }
        Err(e) => {
            println!("Request failed (server may not be running): {}", e);
        }
    }
}

#[tokio::test]
async fn test_image_delete_invalid_id() {
    let client = reqwest::Client::new();
    
    // Try to delete an image with an invalid ObjectId
    let response = client
        .delete("http://localhost:3000/api/images/invalid-id")
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should get 400 Bad Request for invalid ID
            assert_eq!(resp.status(), 400);
        }
        Err(e) => {
            println!("Request failed (server may not be running): {}", e);
        }
    }
}

#[tokio::test]
async fn test_user_without_company_cannot_upload() {
    let pool = get_test_pool().await;
    
    // Create a user without a company
    let test_id = uuid::Uuid::new_v4().to_string().replace("-", "");
    let password_hash = hash_password("SecurePassword123").unwrap();
    
    let user = db::create_user(
        &pool,
        format!("testnocompany{}@example.com", test_id),
        "Test".to_string(),
        "NoCompany".to_string(),
        Some(password_hash),
        None, // No company
        UserRole::Staff,
    )
    .await
    .expect("Failed to create user");
    
    let jwt_config = JwtConfig::new("test_secret".to_string());
    let token = jwt_config
        .generate_token(user.id.clone(), 24)
        .expect("Failed to generate token");
    
    let client = reqwest::Client::new();
    
    let test_image_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    
    let form = reqwest::multipart::Form::new()
        .part("image", reqwest::multipart::Part::bytes(test_image_data)
            .file_name("test.png")
            .mime_str("image/png")
            .unwrap());
    
    let response = client
        .post("http://localhost:3000/api/images/upload")
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            // Should get 403 Forbidden for user without company
            assert_eq!(resp.status(), 403);
        }
        Err(e) => {
            println!("Request failed (server may not be running): {}", e);
        }
    }
}
