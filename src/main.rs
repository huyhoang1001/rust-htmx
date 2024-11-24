mod views;
mod data;

use std::env;
// region:    --- Modules
use std::sync::Arc;
use tower_http::services::ServeDir;
use axum::{Router, routing::get};
use axum::routing::post;
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
}


#[tokio::main]
async fn main() {
	// its subdirectories

	tracing_subscriber::fmt()
		.without_time() // For early local development.
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	// let mut posts = vec![];
	let posts = Arc::new(Mutex::new(vec![]));

	let mut join_set = JoinSet::new();
	let post_data_source = PostDataSource::new(&mut join_set, &posts);

	let app_state = AppState {
		post_receiver: post_data_source.receiver,
		posts,
	};
	let current_dir = env::current_dir().unwrap();
	let lib_path = current_dir.join("src/lib");
	println!("abc {:?}", lib_path);

	// -- Define Routes
	let app = Router::new()
		.route("/", get(views::home::home))
		.route("/home", get(views::home::home))
		.route("/home/sse", get(views::home::home_sse))
		.route("/home", post(views::home::create_post))
		.nest_service("/lib", ServeDir::new(lib_path))
		.with_state(app_state);

	// region:    --- Start Server
	// Note: For this block, ok to unwrap.
	let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
	info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
	axum::serve(listener, app.into_make_service())
		.await
		.unwrap();
	// endregion: --- Start Server
}