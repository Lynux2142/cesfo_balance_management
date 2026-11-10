use axum::{
    extract::{State, Query},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use cesfo_balance_management::{DatabaseService, CesfoClient, Credentials, models::AccountProfile};
use serde::Deserialize;
use std::{env, sync::Arc};
use tera::Tera;
use tokio_cron_scheduler::{Job, JobScheduler};
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
    println!("Starting CESFO Web Server & Scheduler...");

    let mongo_url = env::var("MONGODB_URL").expect("MANDATORY ERROR: MONGODB_URL is not set.");
    let mongo_port = env::var("MONGODB_PORT").expect("MANDATORY ERROR: MONGODB_PORT is not set.");
    let mongo_user = env::var("MONGODB_USER").expect("MANDATORY ERROR: MONGODB_USER is not set.");
    let mongo_password = env::var("MONGODB_PASSWORD").expect("MANDATORY ERROR: MONGODB_PASSWORD is not set.");
    let cron_schedule = env::var("CRON_SCHEDULE").expect("MANDATORY ERROR: CRON_SCHEDULE is not set.");

    let mongo_uri = format!("mongodb://{}:{}@{}:{}", mongo_user, mongo_password, mongo_url, mongo_port);
    let db_service = DatabaseService::new(&mongo_uri, "cesfo_db").await?;
    println!("Connected to MongoDB successfully.");

    // Setup background cron scheduler looping through all accounts
    let scheduler = JobScheduler::new().await?;
    let db_clone = db_service.clone();

    scheduler.add(
        Job::new_async(&cron_schedule, move |_uuid, _lock| {
            let db = db_clone.clone();
            Box::pin(async move {
                println!("[CRON] Triggered automated scrape cycle for all registered accounts...");

                let accounts = match db.get_all_accounts().await.map_err(|e| e.to_string()) {
                    Ok(accs) => accs,
                    Err(e) => {
                        eprintln!("[CRON ERROR] Failed to fetch accounts from database: {}", e);
                        return;
                    }
                };

                if accounts.is_empty() {
                    println!("[CRON] No accounts registered in the database yet.");
                    return;
                }

                for acc in accounts {
                    println!("[CRON] Scraping account: {} {} (Badge: {})", acc.first_name, acc.last_name, acc.badge_number);

                    let creds = Credentials {
                        badge_div: acc.badge_div.clone(),
                        badge_number: acc.badge_number.clone(),
                        badge_nom: format!("{} {}", acc.last_name, acc.first_name).to_lowercase(),
                    };

                    let client = match CesfoClient::new() {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("[CRON ERROR] Failed to create CESFO client: {}", e);
                            continue;
                        }
                    };

                    let html_body = match client.authenticate(&creds).await {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("[CRON ERROR] Authentication failed for badge {}: {}", creds.badge_number, e);
                            continue;
                        }
                    };

                    let mut info = match CesfoClient::parse_account_info(&html_body) {
                        Ok(i) => i,
                        Err(e) => {
                            eprintln!("[CRON ERROR] Failed to parse HTML for badge {}: {}", creds.badge_number, e);
                            continue;
                        }
                    };

                    info.badge_number = creds.badge_number.clone();

                    if let Err(e) = db.save_account_info(info).await.map_err(|e| e.to_string()) {
                        eprintln!("[CRON ERROR] Failed to save info for badge {}: {}", creds.badge_number, e);
                    } else {
                        println!("[CRON SUCCESS] Updated balance for badge {}", creds.badge_number);
                    }
                }
            })
        })?
    ).await?;

    scheduler.start().await?;
    println!("[SCHEDULER] Cron loop is active with schedule: '{}'", cron_schedule);

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
            Redirect::to(&format!("/balances?badge={}", account.badge_number)).into_response()
        }
        Ok(None) => {
            let mut ctx = tera::Context::new();
            ctx.insert("error", "Account not found.");
            match state.templates.render("login.html", &ctx) {
                Ok(html) => Html(html).into_response(),
                Err(e) => Html(format!("Template Error: {}", e)).into_response(),
            }
        }
        Err(_) => {
            let mut ctx = tera::Context::new();
            ctx.insert("error", "An internal error occurred.");
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

#[axum::debug_handler]
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

    if let Err(e) = state.db_service.create_account(profile).await {
        println!("[REGISTER FAILED] Failed registration for badge {}: {}", form.badge_number, e);
        let mut ctx = tera::Context::new();
        ctx.insert("error", "Registration failed. Badge number might already exist.");
        return match state.templates.render("register.html", &ctx) {
            Ok(html) => Html(html).into_response(),
            Err(_) => Html(String::from("Template Error")).into_response(),
        };
    }

    println!("[REGISTER SUCCESS] New account created for '{} {}' (Badge: {})", form.first_name, form.last_name, form.badge_number);

    let creds = Credentials {
        badge_div: form.badge_div.clone(),
        badge_number: form.badge_number.clone(),
        badge_nom: format!("{} {}", form.last_name, form.first_name).to_lowercase(),
    };

    let scrape_outcome = async {
        let client = CesfoClient::new().map_err(|e| e.to_string())?;
        let html_body = client.authenticate(&creds).await.map_err(|e| e.to_string())?;
        let mut info = CesfoClient::parse_account_info(&html_body).map_err(|e| e.to_string())?;
        info.badge_number = form.badge_number.clone();
        Ok::<_, String>(info)
    }.await;

    match scrape_outcome {
        Ok(info) => {
            if let Err(e) = state.db_service.save_account_info(info).await {
                eprintln!("[REGISTER SCRAPE ERROR] Failed to save initial balance: {}", e);
            } else {
                println!("[REGISTER SCRAPE SUCCESS] Initial balance saved.");
            }
        }
        Err(e) => {
            eprintln!("[REGISTER SCRAPE ERROR] {}", e);
        }
    }

    Redirect::to(&format!("/balances?badge={}", form.badge_number)).into_response()
}

async fn show_balances(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BalanceQuery>,
) -> impl IntoResponse {
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
