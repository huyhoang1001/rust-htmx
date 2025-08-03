use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sse_rust_htmx::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_and_update_post() {
    // Setup test app
    let posts = Arc::new(Mutex::new(vec![]));
    let mut join_set = tokio::task::JoinSet::new();
    let post_data_source = sse_rust_htmx::data::posts_datasource::PostDataSource::new(&mut join_set, &posts);
    
    let app_state = sse_rust_htmx::AppState {
        post_receiver: post_data_source.receiver,
        posts: posts.clone(),
    };

    let app = Router::new()
        .route("/", axum::routing::get(sse_rust_htmx::controller::home::home))
        .route("/home", axum::routing::get(sse_rust_htmx::controller::home::home))
        .route("/home/sse", axum::routing::get(sse_rust_htmx::controller::home::home_sse))
        .route("/home", axum::routing::post(sse_rust_htmx::controller::home::create_post))
        .route("/posts/{id}/edit", axum::routing::get(sse_rust_htmx::controller::home::edit_post))
        .route("/posts/{id}", axum::routing::put(sse_rust_htmx::controller::home::update_post))
        .with_state(app_state);

    // Create a post first
    let create_request = Request::builder()
        .method("POST")
        .uri("/home")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=testuser&message=Original message"))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);

    // Test getting edit form
    let edit_request = Request::builder()
        .method("GET")
        .uri("/posts/0/edit")
        .body(Body::empty())
        .unwrap();

    let edit_response = app.clone().oneshot(edit_request).await.unwrap();
    assert_eq!(edit_response.status(), StatusCode::OK);

    // Test updating the post
    let update_request = Request::builder()
        .method("PUT")
        .uri("/posts/0")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=testuser&message=Updated message"))
        .unwrap();

    let update_response = app.oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);

    // Verify the post was updated
    let posts_lock = posts.lock().await;
    assert_eq!(posts_lock.len(), 1);
    assert_eq!(posts_lock[0].message, "Updated message");
}

#[tokio::test]
async fn test_update_nonexistent_post() {
    // Setup test app
    let posts = Arc::new(Mutex::new(vec![]));
    let mut join_set = tokio::task::JoinSet::new();
    let post_data_source = sse_rust_htmx::data::posts_datasource::PostDataSource::new(&mut join_set, &posts);
    
    let app_state = sse_rust_htmx::AppState {
        post_receiver: post_data_source.receiver,
        posts,
    };

    let app = Router::new()
        .route("/", axum::routing::get(sse_rust_htmx::controller::home::home))
        .route("/home", axum::routing::get(sse_rust_htmx::controller::home::home))
        .route("/home/sse", axum::routing::get(sse_rust_htmx::controller::home::home_sse))
        .route("/home", axum::routing::post(sse_rust_htmx::controller::home::create_post))
        .route("/posts/{id}/edit", axum::routing::get(sse_rust_htmx::controller::home::edit_post))
        .route("/posts/{id}", axum::routing::put(sse_rust_htmx::controller::home::update_post))
        .with_state(app_state);

    // Try to update a non-existent post
    let update_request = Request::builder()
        .method("PUT")
        .uri("/posts/999")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=testuser&message=Updated message"))
        .unwrap();

    let update_response = app.oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_with_empty_message() {
    // Setup test app
    let posts = Arc::new(Mutex::new(vec![]));
    let mut join_set = tokio::task::JoinSet::new();
    let post_data_source = sse_rust_htmx::data::posts_datasource::PostDataSource::new(&mut join_set, &posts);
    
    let app_state = sse_rust_htmx::AppState {
        post_receiver: post_data_source.receiver,
        posts: posts.clone(),
    };

    let app = Router::new()
        .route("/", axum::routing::get(sse_rust_htmx::controller::home::home))
        .route("/home", axum::routing::get(sse_rust_htmx::controller::home::home))
        .route("/home/sse", axum::routing::get(sse_rust_htmx::controller::home::home_sse))
        .route("/home", axum::routing::post(sse_rust_htmx::controller::home::create_post))
        .route("/posts/{id}/edit", axum::routing::get(sse_rust_htmx::controller::home::edit_post))
        .route("/posts/{id}", axum::routing::put(sse_rust_htmx::controller::home::update_post))
        .with_state(app_state);

    // Create a post first
    let create_request = Request::builder()
        .method("POST")
        .uri("/home")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=testuser&message=Original message"))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);

    // Try to update with empty message
    let update_request = Request::builder()
        .method("PUT")
        .uri("/posts/0")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=testuser&message="))
        .unwrap();

    let update_response = app.oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::BAD_REQUEST);
}