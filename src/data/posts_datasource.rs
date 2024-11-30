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
    //Arc is used to share ownership of the posts vector across threads/tasks safely.
    //Mutex is used to allow mutable access to the vector.
    // Since the async task needs to mutate the posts vector, you lock it with lock().await before cloning.
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