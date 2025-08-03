pub mod views;
pub mod data;
pub mod controller;

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::data::model::Post;

#[derive(Clone)]
pub struct AppState {
    pub posts: Arc<Mutex<Vec<Post>>>,
    pub post_receiver: tokio::sync::watch::Receiver<Vec<Post>>,
    pub next_post_id: Arc<Mutex<u64>>,
}