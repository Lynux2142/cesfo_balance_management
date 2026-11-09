use cesfo_balance_management::{CesfoClient, DatabaseService, Credentials};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin scraper -- <badge_number>");
        std::process::exit(1);
    }

    let badge_number = &args[1];
    println!("Starting CESFO workflow for badge: {}...", badge_number);

    let db_service = DatabaseService::new(
        "mongodb://lguiller:Tekken18@localhost:27017/",
        "cesfo_db",
    ).await?;
    println!("Connected to MongoDB successfully.");

    let account_profile = match db_service.get_account_by_badge(badge_number).await? {
        Some(profile) => profile,
        None => {
            eprintln!("Error: No account found with badge number '{}' in the database. Please register on the web server first.", badge_number);
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

    let mut account_info = CesfoClient::parse_account_info(&html_body)?;
    
    account_info.badge_number = creds.badge_number;

    db_service.save_account_info(account_info).await?;

    Ok(())
}
