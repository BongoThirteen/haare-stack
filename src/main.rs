
use tracing::info;
use tracing_subscriber::{
    registry,
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    fmt::layer
};
use anyhow::Context;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
    serve,
    extract::State,
    Form,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::fs::{read_to_string, write};
use walkdir::WalkDir;
use encre_css::{Config, Preflight, generate};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate;

async fn hello() -> impl IntoResponse {
    let template = HelloTemplate;
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "another-page.html")]
struct AnotherPageTemplate;

async fn another_page() -> impl IntoResponse {
    let template = AnotherPageTemplate;
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "todo-list.html")]
struct TodoList {
    todos: Vec<(usize, String)>,
}

#[derive(Deserialize)]
struct TodoRequest {
    todo: String,
}

async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(todo): Form<TodoRequest>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    todos.push(todo.todo);
    let template = TodoList {
        todos: todos.iter().cloned().enumerate().collect()
    };
    HtmlTemplate(template)
}

async fn todos(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let todos = state.todos.lock().unwrap();
    let template = TodoList {
        todos: todos.iter().cloned().enumerate().collect()
    };
    HtmlTemplate(template)
}

#[derive(Deserialize)]
struct DoneRequest {
    id: usize,
}

async fn remove_todo(
    State(state): State<Arc<AppState>>,
    Form(done): Form<DoneRequest>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();
    todos.remove(done.id);
    let template = TodoList {
        todos: todos.iter().cloned().enumerate().collect()
    };
    HtmlTemplate(template)
}

/// Wrapper for conversion from templates to HTML
struct HtmlTemplate<T>(T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. {err}"),
            )
                .into_response(),
        }
    }
}

async fn hello_api() -> &'static str {
    "Hello from the server!"
}

#[derive(Debug, Default)]
struct AppState {
    todos: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "axum_htmx_askama=debug".into()),
        )
        .with(layer())
        .init();

    info!("building CSS styles");
    let mut config = Config::default();
    config.preflight = Preflight::new_full()
        .font_family_sans(r#""Inter var", ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji""#);
    let mut html = Vec::new();
    for entry in WalkDir::new("templates") {
        let path = entry
            .context("failed to scan HTML files")?
            .into_path();
        if path.is_dir() {
            continue;
        }
        let Ok(text) = read_to_string(&path) else {
            continue;
        };
        html.push(text);
    }
    let css = generate(
        html.iter().map(String::as_str),
        &config,
    );
    
    write("assets/main.css", css).context("failed to write CSS to file")?;

    info!("initializing router");
    let api_router = Router::new()
        .route("/hello", get(hello_api))
        .route("/todos", post(add_todo))
        .route("/todos", get(todos))
        .route("/done", post(remove_todo))
        .with_state(Arc::<AppState>::default());
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(hello))
        .route("/another", get(another_page))
        .nest_service("/assets", ServeDir::new("./assets"));

    let address = SocketAddr::from(([0, 0, 0, 0], 8000));

    info!("listening on {}", address);
    let listener = TcpListener::bind(address)
        .await
        .context("error while opening socket")?;
    serve(listener, router)
        .await
        .context("error while starting server")?;
    Ok(())
}
