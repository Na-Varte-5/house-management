mod common;

use common::{TestServer, TestUser, create_test_user, login_test_user};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn test_register_new_user() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/register", server.base_url))
        .json(&serde_json::json!({
            "email": "newuser@test.com",
            "name": "New User",
            "password": "password123",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let json: Value = response.json().await.expect("Failed to parse response");
    assert!(json["token"].is_string());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Create first user
    create_test_user(&server.pool, TestUser::homeowner()).await;

    // Try to register with same email
    let response = client
        .post(format!("{}/register", server.base_url))
        .json(&serde_json::json!({
            "email": "homeowner@test.com",
            "name": "Duplicate User",
            "password": "password123",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_success() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let user = TestUser::admin();

    create_test_user(&server.pool, user.clone()).await;

    let response = client
        .post(format!("{}/login", server.base_url))
        .json(&serde_json::json!({
            "email": user.email,
            "password": user.password,
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let json: Value = response.json().await.expect("Failed to parse response");
    assert!(json["token"].is_string());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let user = TestUser::admin();

    create_test_user(&server.pool, user.clone()).await;

    let response = client
        .post(format!("{}/login", server.base_url))
        .json(&serde_json::json!({
            "email": user.email,
            "password": "wrongpassword",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/login", server.base_url))
        .json(&serde_json::json!({
            "email": "nobody@test.com",
            "password": "password",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_token_validation() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let user = TestUser::homeowner();

    let user = create_test_user(&server.pool, user).await;
    let token = login_test_user(&client, &server.base_url, &user).await;

    // Try to access a protected endpoint
    let response = client
        .get(format!("{}/users", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_invalid_token_rejected() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/users", server.base_url))
        .bearer_auth("invalid.token.here")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_no_token_rejected() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/users", server.base_url))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rbac_admin_can_list_users() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin = TestUser::admin();

    let admin = create_test_user(&server.pool, admin).await;
    let token = login_test_user(&client, &server.base_url, &admin).await;

    let response = client
        .get(format!("{}/users", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rbac_homeowner_cannot_list_users() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let homeowner = TestUser::homeowner();

    let homeowner = create_test_user(&server.pool, homeowner).await;
    let token = login_test_user(&client, &server.base_url, &homeowner).await;

    let response = client
        .get(format!("{}/users", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_rbac_renter_cannot_list_users() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let renter = TestUser::renter();

    let renter = create_test_user(&server.pool, renter).await;
    let token = login_test_user(&client, &server.base_url, &renter).await;

    let response = client
        .get(format!("{}/users", server.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
