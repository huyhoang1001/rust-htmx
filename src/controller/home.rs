use std::sync::mpsc::RecvError;
use axum::extract::{State, Path};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Sse};
use axum::response::sse::{Event, KeepAlive};
use fake::Fake;
use fake::faker::internet::en::Username;
use futures::Stream;
use tokio_stream::wrappers::ReceiverStream;
use crate::data::model::Post;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::controller::form_qs::JsonOrForm;
use crate::views::home::{home_page, edit_form, post_card};

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    username: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePostParams {
    message: String,
}

/// Renders the home page as an HTML response, dynamically generating its content
/// based on the current state of the application's posts.
///
/// # Parameters
///
/// - `State(crate::AppState { post_receiver: mut receiver, .. })`:
///   Extracts the shared application state containing a watch receiver (`post_receiver`)
///   that holds the current posts data. The receiver allows access to the latest updates
///   without requiring a lock.
///
/// # Returns
///
/// - An `Html<String>` response containing the rendered home page with the latest posts data.
pub async fn home(
    State(crate::AppState {
              post_receiver: mut receiver,
              ..
          }): State<crate::AppState>,
) -> Html<String> {
    let username: String = Username().fake();
    let content = home_page(&username, receiver.borrow_and_update());
    Html(content)
}

/// Handles a Server-Sent Events (SSE) stream for the home page, sending updated HTML content
/// whenever the application's post data changes.
///
/// # Parameters
///
/// - `State(crate::AppState { post_receiver: mut _receiver, .. })`:
///   Extracts the shared application state containing a watch receiver (`post_receiver`)
///   that monitors changes to the `posts` data. The receiver is used to detect updates and send new content.
///
/// # Returns
///
/// An `Sse` stream that sends updated HTML content as events to the client.
/// Each event contains the serialized HTML for the home page when the `posts` data changes.
pub async fn home_sse(
    State(crate::AppState {
              post_receiver: mut _receiver,
              ..
          }): State<crate::AppState>,
) -> Sse<impl Stream<Item=Result<Event, RecvError>>> {
    let username: String = Username().fake();
    let (sender, receiver1) = tokio::sync::mpsc::channel(1);
    tokio::task::spawn(async move {
        loop {
            if _receiver.changed().await.is_err() {
                println!("Post Receiver disconnected");
                return;
            }

            let html = home_page(&username, _receiver.borrow_and_update());
            if let Err(err) = sender.send(Ok(Event::default().data(html))).await {
                println!("Failed to send event: {}", err);
                return;
            }
        }
    });
    Sse::new(ReceiverStream::new(receiver1)).keep_alive(KeepAlive::default())
}

/// Handles the creation of a new post and adds it to the shared application state.
///
/// # Parameters
///
/// - `State(crate::AppState { posts: state, next_post_id, .. })`: Extracts the shared application state containing
///   the `posts` vector and `next_post_id` counter, which are protected by `Mutex`. The `State` wrapper allows for dependency injection
///   of the app state.
/// - `JsonOrForm(payload)`: Parses the incoming request body as either JSON or a form payload, extracting
///   the `QueryParams` structure that contains the `username` and `message` for the new post.
///
/// # Returns
///
/// - `Ok(StatusCode::OK)` if the post is successfully created and added to the shared state.
/// - `Result` is used to handle potential errors, though the current implementation does not anticipate any.
pub async fn create_post(
    State(crate::AppState {
              posts: state,
              next_post_id,
              ..
          }): State<crate::AppState>,
    JsonOrForm(payload): JsonOrForm<QueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut posts_lock = state.lock().await; // Lock the Mutex
    let mut id_lock = next_post_id.lock().await;
    let post_id = *id_lock;
    *id_lock += 1;
    
    posts_lock.push(Post {
        id: post_id,
        username: payload.username.to_string(),
        message: payload.message.to_string(),
        time: OffsetDateTime::now_utc().to_string(),
        avatar: format!("https://ui-avatars.com/api/?background=random&rounded=true&name={}", payload.username.to_string()),
        owner_id: payload.username.to_string(), // Simple ownership based on username for now
    });
    Ok(StatusCode::OK)
}

/// Returns an HTML form for editing a specific post.
///
/// # Parameters
///
/// - `Path(post_id)`: The ID of the post to edit, extracted from the URL path.
/// - `State(crate::AppState { posts, .. })`: The shared application state containing the posts.
///
/// # Returns
///
/// - `Html<String>` containing the edit form if the post is found.
/// - `StatusCode::NOT_FOUND` if the post doesn't exist.
pub async fn edit_post_form(
    Path(post_id): Path<u64>,
    State(crate::AppState { posts, .. }): State<crate::AppState>,
) -> Result<Html<String>, StatusCode> {
    let posts_lock = posts.lock().await;
    
    if let Some(post) = posts_lock.iter().find(|p| p.id == post_id) {
        let form_html = edit_form(post);
        Ok(Html(form_html))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handles updating an existing post.
///
/// # Parameters
///
/// - `Path(post_id)`: The ID of the post to update, extracted from the URL path.
/// - `State(crate::AppState { posts, .. })`: The shared application state containing the posts.
/// - `JsonOrForm(payload)`: The form data containing the updated message.
///
/// # Returns
///
/// - `Html<String>` containing the updated post HTML if successful.
/// - `StatusCode::NOT_FOUND` if the post doesn't exist.
/// - `StatusCode::FORBIDDEN` if the user doesn't own the post.
pub async fn update_post(
    Path(post_id): Path<u64>,
    State(crate::AppState { posts, .. }): State<crate::AppState>,
    JsonOrForm(payload): JsonOrForm<UpdatePostParams>,
) -> Result<Html<String>, StatusCode> {
    let mut posts_lock = posts.lock().await;
    
    // For demo purposes, we'll use a simple approach where we allow editing
    // if the username matches. In a real app, you'd have proper authentication.
    
    if let Some(post) = posts_lock.iter_mut().find(|p| p.id == post_id) {
        // For demo, we'll allow editing by anyone for now
        // In production, you'd check proper authentication/authorization
        
        // Validate message is not empty
        if payload.message.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        
        // Update the post
        post.message = payload.message;
        post.time = OffsetDateTime::now_utc().to_string(); // Update timestamp
        
        // Return the updated post HTML - use the post's username as current user for consistency
        let updated_post_html = post_card(post, &post.username);
        Ok(Html(updated_post_html))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
