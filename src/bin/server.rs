use axum::{
    extract::{State, Query},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use cesfo_balance_management::{DatabaseService, models::AccountProfile};
use serde::Deserialize;
use std::sync::Arc;
use std::env;
use tera::Tera;
use tower_http::services::ServeDir;

struct AppState {
    db_service: DatabaseService,
    templates: Tera,
}

#[derive(Deserialize)]
struct LoginForm {
    first_name: String,
    last_name: String,
}

#[derive(Deserialize)]
struct RegisterForm {
    badge_div: String,
    badge_number: String,
    first_name: String,
    last_name: String,
}

#[derive(Deserialize)]
struct BalanceQuery {
    badge: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting CESFO Multi-Account Web Server...");

    let mongo_url = env::var("MONGODB_URL").expect("MANDATORY ERROR: MONGODB_URL is not set.");
    let mongo_port = env::var("MONGODB_PORT").expect("MANDATORY ERROR: MONGODB_PORT is not set.");
    let mongo_user = env::var("MONGODB_USER").expect("MANDATORY ERROR: MONGODB_USER is not set.");
    let mongo_password = env::var("MONGODB_PASSWORD").expect("MANDATORY ERROR: MONGODB_PASSWORD is not set.");

    let mongo_uri = format!("mongodb://{}:{}@{}:{}", mongo_user, mongo_password, mongo_url, mongo_port);
    let db_service = DatabaseService::new(&mongo_uri, "cesfo_db").await?;
    println!("Connected to MongoDB and indexes verified successfully.");

    let mut templates = Tera::default();
    templates.load_from_glob("templates/**/*.html")?;

    let state = Arc::new(AppState { db_service, templates });

    let app = Router::new()
        .route("/", get(show_login))
        .route("/login", post(handle_login))
        .route("/register", get(show_register))
        .route("/register", post(handle_register))
        .route("/balances", get(show_balances))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Web server running at http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn show_login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ctx = tera::Context::new();
    match state.templates.render("login.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("Template Error: {}", e)).into_response(),
    }
}

async fn handle_login(
    State(state): State<Arc<AppState>>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    match state.db_service.get_account_by_name(&form.first_name, &form.last_name).await {
        Ok(Some(account)) => {
            println!("User '{} {}' logged in (Badge: {})", account.first_name, account.last_name, account.badge_number);
            Redirect::to(&format!("/balances?badge={}", account.badge_number)).into_response()
        }
        Ok(None) => {
            println!("[ERROR] Attempted login for unknown user: '{} {}'", form.first_name, form.last_name);
            let mut ctx = tera::Context::new();
            ctx.insert("error", "Account not found. Please check your name or create an account.");
            match state.templates.render("login.html", &ctx) {
                Ok(html) => Html(html).into_response(),
                Err(e) => Html(format!("Template Error: {}", e)).into_response(),
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Database error during login for '{} {}': {}", form.first_name, form.last_name, e);
            let mut ctx = tera::Context::new();
            ctx.insert("error", "An internal error occurred. Please try again later.");
            match state.templates.render("login.html", &ctx) {
                Ok(html) => Html(html).into_response(),
                Err(e) => Html(format!("Template Error: {}", e)).into_response(),
            }
        }
    }
}

async fn show_register(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ctx = tera::Context::new();
    match state.templates.render("register.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("Template Error: {}", e)).into_response(),
    }
}

async fn handle_register(
    State(state): State<Arc<AppState>>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    let profile = AccountProfile {
        badge_div: form.badge_div.clone(),
        badge_number: form.badge_number.clone(),
        first_name: form.first_name.clone(),
        last_name: form.last_name.clone(),
    };

    match state.db_service.create_account(profile).await {
        Ok(_) => {
            println!("New account created for '{} {}' (Badge: {}, Div: {})", form.first_name, form.last_name, form.badge_number, form.badge_div);
            Redirect::to(&format!("/balances?badge={}", form.badge_number)).into_response()
        }
        Err(e) => {
            println!("[ERROR] Failed registration attempt: first name: {} - last name: {} - badge {}: {}", form.first_name, form.last_name, form.badge_number, e);
            let mut ctx = tera::Context::new();
            ctx.insert("error", "Registration failed. Badge number might already exist.");
            match state.templates.render("register.html", &ctx) {
                Ok(html) => Html(html).into_response(),
                Err(e) => Html(format!("Template Error: {}", e)).into_response(),
            }
        }
    }
}

async fn show_balances(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BalanceQuery>,
) -> impl IntoResponse {
    println!("Balance history accessed for badge: {}", query.badge);

    let records = match state.db_service.get_latest_n_account_infos(&query.badge, 20).await {
        Ok(recs) => recs,
        Err(_) => vec![],
    };

    let profile = match state.db_service.get_account_by_badge(&query.badge).await {
        Ok(Some(p)) => p,
        _ => AccountProfile {
            badge_div: "".to_string(),
            badge_number: query.badge.clone(),
            first_name: "User".to_string(),
            last_name: "".to_string(),
        },
    };

    let mut ctx = tera::Context::new();
    ctx.insert("records", &records);
    ctx.insert("profile", &profile);

    match state.templates.render("balances.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("Template Error: {}", e)).into_response(),
    }
}
