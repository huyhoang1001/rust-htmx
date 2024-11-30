# SSE-Based Real-Time Web Application with RUST and HTMX

This project implements a real-time web application using Rust and the Axum framework. It showcases how to build a Server-Sent Events (SSE) feature, dynamic HTML rendering, and a shared state mechanism for handling posts and user interactions.

---

## Features

- **Real-Time Updates**:  
  Dynamically updated home page using SSE and reactive state management.

- **State Management**:  
  Centralized state handling using `tokio::sync::watch` for efficient data sharing across tasks.

- **Dynamic HTML Rendering**:  
  HTML content dynamically generated based on real-time data and personalized user experiences.

- **Concurrency with Tokio**:  
  Handles tasks efficiently using asynchronous programming with Tokio.

---

## Project Structure

### 1. **Endpoints**
#### `home_sse`
Handles the Server-Sent Events (SSE) connection.  
Real-time updates are sent to the client when the application's `posts` data changes.

#### `home`
Renders the home page as HTML, dynamically generating its content based on the current state of the application's posts.

#### `create_post`
Handles incoming requests to create a new post. Updates the application state and appends the new post to the shared `posts` collection.

---

### 2. **Key Functions**

#### `home_sse`

- **Purpose**: Sends real-time updates to the client whenever new posts are added or modified.
- **Returns**: An SSE stream with HTML updates.

#### `home`

- **Purpose**: Renders the home page with the latest posts, personalized for the user.
- **Returns**: HTML response dynamically created based on the application state.

#### `create_post`

- **Purpose**: Processes new post submissions, appending them to the shared `posts` state.
- **Returns**: An HTTP status code indicating success or failure.

---

### 3. **Core Components**

#### `AppState`
The centralized application state, which includes:
- `post_receiver`: A `tokio::sync::watch::Receiver` for monitoring post updates.
- `posts`: Shared collection of posts (`Arc<Mutex<Vec<Post>>>`).

#### `Post`
A data structure representing a single post with fields:
- `username`: Name of the user who created the post.
- `message`: Content of the post.
- `time`: Timestamp of post creation.
- `avatar`: URL for the user's avatar image.

---

## How to Run
Clone the repository:

```bash
git clone https://github.com/your-repo-url.git
cd your-project-folder
```
### Install Rust:
Follow the instructions at rustup.rs.

### Add dependencies:
Add required crates to your Cargo.toml file, such as axum, tokio, time, and fake.

### Run the application:

```bash
cargo run
```
Visit the endpoints:

Home page: http://localhost:8080

### Dependencies
- Axum: Web framework for building HTTP APIs.
- Tokio: Asynchronous runtime for handling tasks.
- Fake: Library for generating random test data.
- Time: Date and time utilities for Rust.
- Serde: Serialization and deserialization of data.
