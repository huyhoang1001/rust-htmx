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

