mod common;

use common::{TestServer, TestUser, create_and_login_user};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn test_admin_can_create_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    let response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "Test Building",
            "address": "123 Test St",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let json: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(json["name"], "Test Building");
    assert_eq!(json["address"], "123 Test St");
    assert!(json["id"].is_number());
}

#[tokio::test]
async fn test_manager_can_create_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let manager =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::manager()).await;

    let response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(manager.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "Manager Building",
            "address": "456 Manager Ave",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_homeowner_cannot_create_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let homeowner = create_and_login_user(
        &server.pool,
        &client,
        &server.base_url,
        TestUser::homeowner(),
    )
    .await;

    let response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(homeowner.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "Homeowner Building",
            "address": "789 Homeowner Rd",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_buildings() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create some buildings
    for i in 1..=3 {
        client
            .post(format!("{}/buildings", server.base_url))
            .bearer_auth(admin.token.as_ref().unwrap())
            .json(&serde_json::json!({
                "name": format!("Building {}", i),
                "address": format!("{} Test St", i),
            }))
            .send()
            .await
            .expect("Failed to create building");
    }

    // List buildings
    let response = client
        .get(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let json: Value = response.json().await.expect("Failed to parse response");
    let buildings = json.as_array().expect("Expected array");
    assert_eq!(buildings.len(), 3);
}

#[tokio::test]
async fn test_update_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create building
    let create_response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "Old Name",
            "address": "Old Address",
        }))
        .send()
        .await
        .expect("Failed to create building");

    let building: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let building_id = building["id"].as_u64().expect("No ID");

    // Update building
    let response = client
        .put(format!("{}/buildings/{}", server.base_url, building_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "New Name",
            "address": "New Address",
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let updated: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(updated["name"], "New Name");
    assert_eq!(updated["address"], "New Address");
}

#[tokio::test]
async fn test_soft_delete_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create building
    let create_response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "To Delete",
            "address": "Delete St",
        }))
        .send()
        .await
        .expect("Failed to create building");

    let building: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let building_id = building["id"].as_u64().expect("No ID");

    // Delete building
    let response = client
        .delete(format!("{}/buildings/{}", server.base_url, building_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    // Verify not in active list
    let list_response = client
        .get(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to list buildings");

    let buildings: Value = list_response
        .json()
        .await
        .expect("Failed to parse response");
    let buildings_array = buildings.as_array().expect("Expected array");
    assert!(!buildings_array.iter().any(|b| b["id"] == building_id));
}

#[tokio::test]
async fn test_restore_deleted_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create and delete building
    let create_response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "name": "To Restore",
            "address": "Restore St",
        }))
        .send()
        .await
        .expect("Failed to create building");

    let building: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let building_id = building["id"].as_u64().expect("No ID");

    client
        .delete(format!("{}/buildings/{}", server.base_url, building_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to delete building");

    // Restore building
    let response = client
        .post(format!(
            "{}/buildings/{}/restore",
            server.base_url, building_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    // Verify back in active list
    let list_response = client
        .get(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to list buildings");

    let buildings: Value = list_response
        .json()
        .await
        .expect("Failed to parse response");
    let buildings_array = buildings.as_array().expect("Expected array");
    assert!(buildings_array.iter().any(|b| b["id"] == building_id));
}

#[tokio::test]
async fn test_create_apartment() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create building first
    let building_response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
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
    let response = client
        .post(format!("{}/apartments", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "building_id": building_id,
            "unit_number": "101",
            "floor": 1,
            "size_sq_m": 75.5,
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let apartment: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(apartment["unit_number"], "101");
    assert_eq!(apartment["floor"], 1);
}

#[tokio::test]
async fn test_list_apartments_for_building() {
    let server = TestServer::start().await;
    let client = reqwest::Client::new();
    let admin =
        create_and_login_user(&server.pool, &client, &server.base_url, TestUser::admin()).await;

    // Create building
    let building_response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
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

    // Create apartments
    for i in 1..=3 {
        client
            .post(format!("{}/apartments", server.base_url))
            .bearer_auth(admin.token.as_ref().unwrap())
            .json(&serde_json::json!({
                "building_id": building_id,
                "unit_number": format!("10{}", i),
                "floor": 1,
                "size_sq_m": 75.0,
            }))
            .send()
            .await
            .expect("Failed to create apartment");
    }

    // List apartments
    let response = client
        .get(format!(
            "{}/buildings/{}/apartments",
            server.base_url, building_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);
    let apartments: Value = response.json().await.expect("Failed to parse response");
    let apartments_array = apartments.as_array().expect("Expected array");
    assert_eq!(apartments_array.len(), 3);
}

#[tokio::test]
async fn test_apartment_owner_assignment() {
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

    // Create building and apartment
    let building_response = client
        .post(format!("{}/buildings", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
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

    let apartment_response = client
        .post(format!("{}/apartments", server.base_url))
        .bearer_auth(admin.token.as_ref().unwrap())
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

    // Assign owner
    let response = client
        .post(format!(
            "{}/apartments/{}/owners",
            server.base_url, apartment_id
        ))
        .bearer_auth(admin.token.as_ref().unwrap())
        .json(&serde_json::json!({
            "user_id": homeowner.id,
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert!(response.status().is_success());

    // Verify assignment
    let get_response = client
        .get(format!("{}/apartments/{}", server.base_url, apartment_id))
        .bearer_auth(admin.token.as_ref().unwrap())
        .send()
        .await
        .expect("Failed to get apartment");

    let apartment_with_owners: Value = get_response.json().await.expect("Failed to parse response");
    let owners = apartment_with_owners["owners"]
        .as_array()
        .expect("Expected owners array");
    assert_eq!(owners.len(), 1);
    assert_eq!(owners[0]["id"], homeowner.id);
}
