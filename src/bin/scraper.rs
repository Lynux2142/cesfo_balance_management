use cesfo_balance_management::{CesfoClient, DatabaseService, Credentials};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mongo_url = env::var("MONGODB_URL").expect("MANDATORY ERROR: MONGODB_URL is not set.");
    let mongo_port = env::var("MONGODB_PORT").expect("MANDATORY ERROR: MONGODB_PORT is not set.");
    let mongo_user = env::var("MONGODB_USER").expect("MANDATORY ERROR: MONGODB_USER is not set.");
    let mongo_password = env::var("MONGODB_PASSWORD").expect("MANDATORY ERROR: MONGODB_PASSWORD is not set.");

    let mongo_uri = format!("mongodb://{}:{}@{}:{}", mongo_user, mongo_password, mongo_url, mongo_port);
    let db_service = DatabaseService::new(&mongo_uri, "cesfo_db").await?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: Please provide a badge number as an argument.");
        std::process::exit(1);
    }

    let badge_number = &args[1];
    println!("[SCRAPER] Starting CESFO scraping for: {}...", badge_number);

    let account_profile = match db_service.get_account_by_badge(badge_number).await? {
        Some(profile) => profile,
        None => {
            eprintln!("[SCRAPER ERROR] No account found with badge number '{}'.", badge_number);
            std::process::exit(1);
        }
    };

    let creds = Credentials {
        badge_div: account_profile.badge_div,
        badge_number: account_profile.badge_number.clone(),
        badge_nom: format!("{} {}", account_profile.last_name, account_profile.first_name).to_lowercase(),
    };

    let client = CesfoClient::new()?;
    let html_body = client.authenticate(&creds).await?;
    let mut info = CesfoClient::parse_account_info(&html_body)?;
    info.badge_number = creds.badge_number;

    db_service.save_account_info(info).await?;
    println!("[SCRAPER SUCCESS] Successfully scraped and saved badge {}.", badge_number);

    Ok(())
}
