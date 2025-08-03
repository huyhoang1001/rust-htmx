use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

// Import the necessary modules from the main application
use sse_rust_htmx::{AppState, controller::home::*};
use sse_rust_htmx::data::model::Post;
use sse_rust_htmx::data::posts_datasource::PostDataSource;

#[tokio::test]
async fn test_create_and_update_post() {
    // Setup test app state
    let posts = Arc::new(Mutex::new(vec![]));
    let next_post_id = Arc::new(Mutex::new(1u64));
    let mut join_set = tokio::task::JoinSet::new();
    let post_data_source = PostDataSource::new(&mut join_set, &posts);

    let app_state = AppState {
        posts: posts.clone(),
        post_receiver: post_data_source.receiver,
        next_post_id: next_post_id.clone(),
    };

    // Create a test router
    let app = Router::new()
        .route("/home", axum::routing::post(create_post))
        .route("/posts/:id", axum::routing::put(update_post))
        .route("/posts/:id/edit", axum::routing::get(edit_post_form))
        .with_state(app_state);

    // Test creating a post
    let create_request = Request::builder()
        .method("POST")
        .uri("/home")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=testuser&message=Hello World"))
        .unwrap();

    let response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify post was created
    let posts_lock = posts.lock().await;
    assert_eq!(posts_lock.len(), 1);
    assert_eq!(posts_lock[0].message, "Hello World");
    assert_eq!(posts_lock[0].username, "testuser");
    assert_eq!(posts_lock[0].id, 1);
    drop(posts_lock);

    // Test getting edit form
    let edit_form_request = Request::builder()
        .method("GET")
        .uri("/posts/1/edit")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(edit_form_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test updating the post
    let update_request = Request::builder()
        .method("PUT")
        .uri("/posts/1")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("message=Updated Hello World"))
        .unwrap();

    let response = app.clone().oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify post was updated
    let posts_lock = posts.lock().await;
    assert_eq!(posts_lock.len(), 1);
    assert_eq!(posts_lock[0].message, "Updated Hello World");
    assert_eq!(posts_lock[0].username, "testuser");
    assert_eq!(posts_lock[0].id, 1);
}

#[tokio::test]
async fn test_update_nonexistent_post() {
    // Setup test app state
    let posts = Arc::new(Mutex::new(vec![]));
    let next_post_id = Arc::new(Mutex::new(1u64));
    let mut join_set = tokio::task::JoinSet::new();
    let post_data_source = PostDataSource::new(&mut join_set, &posts);

    let app_state = AppState {
        posts,
        post_receiver: post_data_source.receiver,
        next_post_id,
    };

    let app = Router::new()
        .route("/posts/:id", axum::routing::put(update_post))
        .with_state(app_state);

    // Test updating a non-existent post
    let update_request = Request::builder()
        .method("PUT")
        .uri("/posts/999")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("message=This should fail"))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_with_empty_message() {
    // Setup test app state with one post
    let posts = Arc::new(Mutex::new(vec![Post {
        id: 1,
        username: "testuser".to_string(),
        message: "Original message".to_string(),
        time: "2023-01-01T00:00:00Z".to_string(),
        avatar: "https://example.com/avatar.jpg".to_string(),
        owner_id: "testuser".to_string(),
    }]));
    let next_post_id = Arc::new(Mutex::new(2u64));
    let mut join_set = tokio::task::JoinSet::new();
    let post_data_source = PostDataSource::new(&mut join_set, &posts);

    let app_state = AppState {
        posts,
        post_receiver: post_data_source.receiver,
        next_post_id,
    };

    let app = Router::new()
        .route("/posts/:id", axum::routing::put(update_post))
        .with_state(app_state);

    // Test updating with empty message
    let update_request = Request::builder()
        .method("PUT")
        .uri("/posts/1")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("message="))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}