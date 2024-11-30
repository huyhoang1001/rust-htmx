use html_node::{html, text, Node};
use crate::views::layout;
use tokio::sync::watch::Ref;
use crate::data::model::Post;

/// Generates the HTML content for the home page.
///
/// # Parameters
///
/// - `username`: A string slice representing the username of the current user.
/// - `posts`: A reference to a vector of `Post` instances wrapped in `Ref`.
///   This contains the posts to be displayed on the home page.
///
/// # Returns
///
/// A `String` containing the generated HTML content for the home page.
pub fn home_page(username: &str, posts: Ref<Vec<Post>>) -> String {
    println!("posts {:?}", posts.clone());
    let html_content = layout(html! {
        <body>
            <div class="content">
                <div hx-ext="morph, sse"
                    sse-connect="http://localhost:8080/home/sse"
                    sse-swap="message"
                    hx-select=".wrapper"
                    hx-include="data-query"
                    hx-swap=r#"morph:{ignoreActiveValue:true,morphStyle:'innerHTML'}"#>

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
                                                data-query
                                                type="hidden"
                                                class="form-control"
                                                name="username"
                                                readonly="true"
                                                value={username}
                                            />
                                            <div class="mb-3 row">
                                                <label for="txtMessage"> "Message:"  </label>
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
                                                                    <img class="me-4" src={text!("{}", post.avatar.to_string())} width="108" />
                                                                <div>
                                                                <h5 class="card-title text-muted">
                                                                    {text!("{}: ", post.username)}
                                                                    <small> {text!("{}", post.time)} </small>
                                                                </h5>
                                                                <div class="card-text lead mb-2">
                                                                    {text!("{}", post.message.to_string())}
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
