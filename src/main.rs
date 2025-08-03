mod views;
mod data;
mod controller;

use std::env;
// region:    --- Modules
use std::sync::Arc;
use tower_http::services::ServeDir;
use axum::{Router, routing::get};
use axum::routing::{post, put};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::info;
use tracing_subscriber::EnvFilter;
use crate::data::model::Post;
use crate::data::posts_datasource::PostDataSource;
// endregion: --- Modules

#[derive(Clone)]
struct AppState {
    posts: Arc<Mutex<Vec<Post>>>,
    post_receiver: tokio::sync::watch::Receiver<Vec<Post>>,
    next_post_id: Arc<Mutex<u64>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .without_time() // For early local development.
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let posts = Arc::new(Mutex::new(vec![]));
    let next_post_id = Arc::new(Mutex::new(1u64));
    let mut join_set = JoinSet::new();
    let post_data_source = PostDataSource::new(&mut join_set, &posts);

    let app_state = AppState {
        post_receiver: post_data_source.receiver,
        posts,
        next_post_id,
    };
    let current_dir = env::current_dir().unwrap();
    let lib_path = current_dir.join("src/lib");

    // -- Define Routes
    let app = Router::new()
        .route("/", get(controller::home::home))
        .route("/home", get(controller::home::home))
        .route("/home/sse", get(controller::home::home_sse))
        .route("/home", post(controller::home::create_post))
        .route("/posts/:id/edit", get(controller::home::edit_post_form))
        .route("/posts/:id", put(controller::home::update_post))
        .nest_service("/lib", ServeDir::new(lib_path))
        .with_state(app_state);

    // region:    --- Start Server
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
    // endregion: --- Start Server
}
