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
use crate::views::home::{home_page, edit_form, post_html};
use html_node::{html, text};

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    username: String,
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
/// - `State(crate::AppState { posts: state, .. })`: Extracts the shared application state containing
///   the `posts` vector, which is protected by a `Mutex`. The `State` wrapper allows for dependency injection
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
              ..
          }): State<crate::AppState>,
    JsonOrForm(payload): JsonOrForm<QueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut posts_lock = state.lock().await; // Lock the Mutex
    let id = posts_lock.len(); // Simple ID assignment based on current length
    posts_lock.push(Post {
        id,
        username: payload.username.to_string(),
        message: payload.message.to_string(),
        time: OffsetDateTime::now_utc().to_string(),
        avatar: format!("https://ui-avatars.com/api/?background=random&rounded=true&name= {}", payload.username.to_string()),
    });
    Ok(StatusCode::OK)
}

/// Handles GET /posts/:id/edit - returns HTML form with post data for editing
///
/// # Parameters
///
/// - `Path(id)`: Extracts the post ID from the URL path
/// - `State(crate::AppState { posts: state, .. })`: Extracts the shared application state
///
/// # Returns
///
/// - `Html<String>` containing the edit form if the post exists
/// - `StatusCode::NOT_FOUND` if the post doesn't exist
pub async fn edit_post(
    Path(id): Path<usize>,
    State(crate::AppState {
              posts: state,
              ..
          }): State<crate::AppState>,
) -> Result<Html<String>, StatusCode> {
    let posts_lock = state.lock().await;
    
    if let Some(post) = posts_lock.iter().find(|p| p.id == id) {
        let form_html = edit_form(post);
        Ok(Html(form_html))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handles PUT /posts/:id - updates an existing post
///
/// # Parameters
///
/// - `Path(id)`: Extracts the post ID from the URL path
/// - `State(crate::AppState { posts: state, .. })`: Extracts the shared application state
/// - `JsonOrForm(payload)`: Parses the incoming request body
///
/// # Returns
///
/// - `Html<String>` containing the updated post HTML if successful
/// - `StatusCode::NOT_FOUND` if the post doesn't exist
/// - `StatusCode::BAD_REQUEST` if validation fails
pub async fn update_post(
    Path(id): Path<usize>,
    State(crate::AppState {
              posts: state,
              ..
          }): State<crate::AppState>,
    JsonOrForm(payload): JsonOrForm<QueryParams>,
) -> Result<Html<String>, StatusCode> {
    // Validate input
    if payload.message.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut posts_lock = state.lock().await;
    
    if let Some(post) = posts_lock.iter_mut().find(|p| p.id == id) {
        // Update the post
        post.message = payload.message.to_string();
        post.username = payload.username.to_string();
        // Keep the original time and avatar
        
        // Return the updated post HTML
        let updated_post_html = post_html(post);
        Ok(Html(updated_post_html.to_string()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Handles GET /posts/:id/cancel - cancels editing and returns the original post content
///
/// # Parameters
///
/// - `Path(id)`: Extracts the post ID from the URL path
/// - `State(crate::AppState { posts: state, .. })`: Extracts the shared application state
///
/// # Returns
///
/// - `Html<String>` containing the original post content if the post exists
/// - `StatusCode::NOT_FOUND` if the post doesn't exist
pub async fn cancel_edit_post(
    Path(id): Path<usize>,
    State(crate::AppState {
              posts: state,
              ..
          }): State<crate::AppState>,
) -> Result<Html<String>, StatusCode> {
    let posts_lock = state.lock().await;
    
    if let Some(post) = posts_lock.iter().find(|p| p.id == id) {
        let post_content_html = html! {
            <div class="card-text lead mb-2" id={text!("post-content-{}", post.id)}>
                {text!("{}", post.message.to_string())}
            </div>
        };
        Ok(Html(post_content_html.to_string()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
