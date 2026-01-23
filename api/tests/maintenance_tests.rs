mod common;

use common::{TestServer, TestUser, create_and_login_user};
use reqwest::StatusCode;
use serde_json::Value;

async fn create_test_building_and_apartment(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
) -> (u64, u64) {
    // Create building
    let building_response = client
        .post(format!("{}/buildings", base_url))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "name": "Test Building",
            "address": "123 Test St",
        }))
        .send()
        .await
        .expect("Failed to create building");

    let building: Value = building_response
        .json()
        .await
        .expect("Failed to parse response");
    let building_id = building["id"].as_u64().expect("No building ID");

    // Create apartment
    let apartment_response = client
        .post(format!("{}/apartments", base_url))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "building_id": building_id,
            "unit_number": "101",
            "floor": 1,
            "size_sq_m": 75.0,
        }))
        .send()
        .await
        .expect("Failed to create apartment");

    let apartment: Value = apartment_response
        .json()
        .await
        .expect("Failed to parse response");
    let apartment_id = apartment["id"].as_u64().expect("No apartment ID");

    (building_id, apartment_id)
}

#[tokio::test]
async fn test_homeowner_can_create_maintenance_request() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to homeowner
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create maintenance request
    let response = client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment_id,
            "title": "Leaky Faucet",
            "description": "The kitchen faucet is dripping",
            "priority": "Medium",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let request: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(request["title"], "Leaky Faucet");
    assert_eq!(request["status"], "Open");
}

#[tokio::test]
async fn test_renter_can_create_maintenance_request() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let renter =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::renter()).await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to renter
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": renter.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create maintenance request
    let response = client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(renter.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment_id,
            "title": "Broken Door",
            "description": "The front door won't close properly",
            "priority": "High",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_maintenance_requests() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to homeowner
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create multiple requests
    for i in 1..=3 {
        client
            .post(format!("{}/requests", server.base_url))
            .bearer_auth(homeowner.token.as_ref().unwrap())
            .json(&serde_json::json!({
                "apartment_id": apartment_id,
                "title": format!("Request {}", i),
                "description": format!("Description {}", i),
                "priority": "Medium",
            }))
            .send()
            .await
            .expect("Failed to create request");
    }

    // List requests
    let response = client
        .get(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let requests: Value = response.json().await.expect("Failed to parse response");
    let requests_array = requests.as_array().expect("Expected array");
    assert_eq!(requests_array.len(), 3);
}

#[tokio::test]
async fn test_manager_can_update_request_status() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let manager =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::manager()).await;
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to homeowner
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create maintenance request
    let create_response = client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment_id,
            "title": "Status Update Test",
            "description": "Test status update",
            "priority": "Medium",
        }))
        .send()
        .await
        .expect("Failed to create request");

    let request: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let request_id = request["id"].as_u64().expect("No request ID");

    // Update status
    let response = client
        .put(format!(
            "{}/requests/{}/status",
            server.base_url, request_id
        ))
        .bearer_auth(manager.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "status": "InProgress",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let updated: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(updated["status"], "InProgress");
}

#[tokio::test]
async fn test_homeowner_cannot_update_request_status() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to homeowner
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create maintenance request
    let create_response = client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment_id,
            "title": "Permission Test",
            "description": "Test permission",
            "priority": "Medium",
        }))
        .send()
        .await
        .expect("Failed to create request");

    let request: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let request_id = request["id"].as_u64().expect("No request ID");

    // Try to update status as homeowner
    let response = client
        .put(format!(
            "{}/requests/{}/status",
            server.base_url, request_id
        ))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "status": "Resolved",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_can_assign_request() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let manager =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::manager()).await;
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to homeowner
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create maintenance request
    let create_response = client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment_id,
            "title": "Assignment Test",
            "description": "Test assignment",
            "priority": "High",
        }))
        .send()
        .await
        .expect("Failed to create request");

    let request: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let request_id = request["id"].as_u64().expect("No request ID");

    // Assign to manager
    let response = client
        .post(format!(
            "{}/requests/{}/assign",
            server.base_url, request_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "user_id": manager.id,
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    // Verify assignment
    let get_response = client
        .get(format!("{}/requests/{}", server.base_url, request_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to get request");

    let request_detail: Value = get_response.json().await.expect("Failed to parse response");
    assert_eq!(request_detail["assigned_to"], manager.id);
}

#[tokio::test]
async fn test_get_request_detail() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let (_building_id, apartment_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Assign apartment to homeowner
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create maintenance request
    let create_response = client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment_id,
            "title": "Detail Test",
            "description": "Test getting detail",
            "priority": "Low",
        }))
        .send()
        .await
        .expect("Failed to create request");

    let request: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let request_id = request["id"].as_u64().expect("No request ID");

    // Get request detail
    let response = client
        .get(format!("{}/requests/{}", server.base_url, request_id))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let detail: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(detail["id"], request_id);
    assert_eq!(detail["title"], "Detail Test");
    assert_eq!(detail["description"], "Test getting detail");
    assert_eq!(detail["priority"], "Low");
    assert_eq!(detail["status"], "Open");
}

#[tokio::test]
async fn test_user_cannot_see_other_users_requests() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    let homeowner1 = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let mut homeowner2 = TestUser::homeowner();
    homeowner2.email = "homeowner2@test.com".to_string();
    let homeowner2 =
        create_and_login_user(&server.pool, &client, &server.base_url, homeowner2).await;

    // Create apartments
    let (_building_id, apartment1_id) = create_test_building_and_apartment(
        &client,
        &server.base_url,
        admin.token.as_ref().unwrap(),
    )
    .await;

    // Create second apartment in same building
    let apartment2_response = client
        .post(format!("{}/apartments", &server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "building_id": _building_id,
            "unit_number": "102",
            "floor": 1,
            "size_sq_m": 75.0,
        }))
        .send()
        .await
        .expect("Failed to create apartment");

    let apartment2: Value = apartment2_response
        .json()
        .await
        .expect("Failed to parse response");
    let apartment2_id = apartment2["id"].as_u64().expect("No apartment ID");

    // Assign apartments
    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment1_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner1.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment2_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({"user_id": homeowner2.id}))
        .send()
        .await
        .expect("Failed to assign owner");

    // Create request for homeowner2
    client
        .post(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner2.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "apartment_id": apartment2_id,
            "title": "Homeowner2's Request",
            "description": "Should not be visible to homeowner1",
            "priority": "Medium",
        }))
        .send()
        .await
        .expect("Failed to create request");

    // Homeowner1 lists requests - should not see homeowner2's request
    let response = client
        .get(format!("{}/requests", server.base_url))
        .bearer_auth(homeowner1.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    let requests: Value = response.json().await.expect("Failed to parse response");
    let requests_array = requests.as_array().expect("Expected array");
    assert_eq!(requests_array.len(), 0);
}
