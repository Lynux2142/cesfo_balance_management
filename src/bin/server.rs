use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use cesfo_balance_management::DatabaseService;
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;

struct AppState {
    db_service: DatabaseService,
    templates: Tera,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting CESFO Web Server...");

    let db_service = DatabaseService::new(
        "mongodb://lguiller:Tekken18@localhost:27017/",
        "cesfo_db",
        "balances",
    ).await?;
    println!("Connected to MongoDB successfully.");

    let mut templates = Tera::default();
    templates.load_from_glob("templates/**/*.html")?;

    let state = Arc::new(AppState { db_service, templates });

    let app = Router::new()
        .route("/", get(list_balances))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Web server running at http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_balances(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let records = match state.db_service.get_latest_n_account_infos(20).await {
        Ok(recs) => recs,
        Err(e) => return Html(format!("<h1>Database Error: {}</h1>", e)).into_response(),
    };

    let mut ctx = tera::Context::new();
    ctx.insert("records", &records);

    match state.templates.render("balances.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("<h1>Template Error: {}</h1>", e)).into_response(),
    }
}
