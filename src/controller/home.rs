use std::sync::mpsc::RecvError;
use axum::extract::State;
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
use crate::views::home::home_page;

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
    posts_lock.push(Post {
        username: payload.username.to_string(),
        message: payload.message.to_string(),
        time: OffsetDateTime::now_utc().to_string(),
        avatar: format!("https://ui-avatars.com/api/?background=random&rounded=true&name= {}", payload.username.to_string()),
    });
    Ok(StatusCode::OK)
}

