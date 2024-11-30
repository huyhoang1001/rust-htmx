use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use crate::data::model::Post;

pub struct PostDataSource {
    pub receiver: tokio::sync::watch::Receiver<Vec<Post>>,
}

impl PostDataSource {
    /// Creates a new instance of `PostDataSource`, which monitors changes to a shared
    /// `Vec<Post>` and broadcasts updates to listeners through a `tokio::sync::watch::Receiver`.
    ///
    /// # Parameters
    ///
    /// - `join_set`: A mutable reference to a `JoinSet` that manages asynchronous tasks.
    ///   A task will be spawned to monitor changes to the `posts` vector and send updates.
    /// - `posts`: A reference-counted, thread-safe, asynchronous `Vec<Post>` wrapped in
    ///   `Arc<Mutex<_>>`. This vector represents the data being monitored for changes.
    ///
    /// # Behavior
    ///
    /// This function:
    /// 1. Spawns an asynchronous task to continuously monitor the `posts` vector for changes.
    /// 2. Uses a hash of the `posts` data to detect changes.
    /// 3. Sends updates to the `tokio::sync::watch::Receiver` only when the data changes,
    ///    avoiding redundant updates.
    /// 4. Runs the monitoring loop with a one-second interval between checks to avoid busy-waiting.
    ///
    /// # Returns
    ///
    /// A `PostDataSource` instance that provides a `tokio::sync::watch::Receiver`
    /// to listen for updates to the `posts` vector.
    pub fn new(join_set: &mut JoinSet<anyhow::Error>, posts: &Arc<Mutex<Vec<Post>>>) -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(vec![]);
        let posts_clone = posts.clone();

        // Spawn a task to monitor changes to `posts` and send updates
        join_set.spawn(async move {
            let mut last_hash: u64 = 0; // Track the last sent posts
            loop {
                let mut hasher = DefaultHasher::new();

                let posts_lock = posts_clone.lock().await;
                posts_lock.hash(&mut hasher);
                let hash = hasher.finish();

                // Only send the posts if they have changed since the last send
                if hash != last_hash {
                    sender.send_replace(posts_lock.clone());
                    last_hash = hash; // Update the last sent posts
                }

                // Sleep or wait for a signal to avoid busy-waiting
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });

        PostDataSource { receiver }
    }
}
