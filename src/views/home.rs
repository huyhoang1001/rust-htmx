use std::sync::mpsc::RecvError;
use axum::extract::State;
use axum::response::{Html, Sse};
use axum::response::sse::{Event, KeepAlive};
use fake::Fake;
use html_node::{html, text, Node};
use crate::views::layout;
use fake::faker::internet::en::Username;
use futures::Stream;
use tokio::sync::watch::Ref;
use tokio_stream::wrappers::ReceiverStream;
use crate::data::model::Post;

fn tweet(username: &str, posts: Ref<Vec<Post>>) -> String {
    println!("posts {:?}", posts.clone());
    let html_content = layout(html! {
        <body>
            <div class="content">
                <div hx-ext="morph, sse"
                    sse-connect="http://localhost:8080/home/sse"
                    sse-swap="message"
                    hx-select=".wrapper"
                    hx-swap=r#"morph:innerHTML"#>

                    <div class="wrapper">
                        <nav class="navbar navbar-dark bg-dark shadow-sm py-0">
                            <div class="container-nav">
                                <a class="navbar-brand" href="#"> "htmx-twitter" </a>
                                <span class="navbar-text text-white"> {text!("{}", username)} </span>
                            </div>
                        </nav>

                        <div class="container">
                            <div class="row justify-content-center">
                                <main class="col-10">
                                    <p class="text-center mt-2">
                                        "A Twitter clone in "
                                        <a href="https://htmx.org"> "htmx" </a>
                                        " and Node"
                                    </p>
                                    <div>
                                        <form hx-post="http://localhost:8080/home" hx-swap="none">
                                            <input
                                                type="hidden"
                                                class="form-control"
                                                name="username"
                                                readonly="true"
                                            />
                                            <div class="mb-3 row">
                                                <label for="txtMessage"> "Message:" </label>
                                                <textarea
                                                    id="txtMessage"
                                                    class="form-control"
                                                    rows="3"
                                                    name="message"
                                                    required="true"
                                                > </textarea>
                                            </div>
                                            <div class="d-grid gap-2 col-3 mx-auto mb-3">
                                                <button
                                                    type="submit"
                                                    class="btn btn-primary text-center"
                                                > "Tweet" </button>
                                            </div>
                                        </form>
                                    </div>

                                    {
                                        if posts.is_empty() {
                                            html! {""}
                                        } else {
                                            Node::from(posts.iter().map(|post| {
                                                html! {
                                                    <div>
                                                        <div class="card mb-2 shadow-sm" id="tweet-{{t.id}}">
                                                            <div class="card-body">
                                                                <div class="d-flex">
                                                                    <img class="me-4" src="" width="108" />
                                                                <div>
                                                                <h5 class="card-title text-muted">
                                                                    {text!("{}", post.username)}
                                                                    <small> : time</small>
                                                                </h5>
                                                                <div class="card-text lead mb-2">
                                                                    Htest
                                                                </div>
                                                            </div>
                                                          </div>
                                                        </div>
                                                      </div>
                                                    </div>
                                                }
                                            }))
                                        }
                                    }

                                </main>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </body>
    });
    html_content.to_string()
}
pub async fn home(
    State(crate::AppState {
        post_receiver: mut receiver,
        ..
    }): State<crate::AppState>
) -> Html<String> {
    let username: String = Username().fake();
    let content = tweet(&username, receiver.borrow_and_update());
    Html(content)
}

pub async fn home_sse(
    State(crate::AppState {
        post_receiver: mut _receiver,
        ..
    }): State<crate::AppState>
) -> Sse<impl Stream<Item = Result<Event, RecvError>>> {
    let username: String = Username().fake();
    let (sender, receiver1) = tokio::sync::mpsc::channel(1);
    tokio::task::spawn(async move {
        loop {
            if _receiver.changed().await.is_err() {
                println!("Post Receiver disconnected");
                return;
            }

            let html = tweet(&username, _receiver.borrow_and_update());
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
) {
    let mut posts_lock = state.lock().await; // Lock the Mutex
    posts_lock.push(Post {
        username: "test".parse().unwrap(),
        message: "".to_string(),
        id: "".to_string(),
        retweets: 0,
        likes: 0,
        time: "".to_string(),
        avatar: "".to_string(),
    });  // Modify the vector
}

