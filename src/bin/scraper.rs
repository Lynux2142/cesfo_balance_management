use cesfo_balance_management::{CesfoClient, DatabaseService, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting CESFO workflow...");

    let db_service = DatabaseService::new(
        "mongodb://lguiller:Tekken18@localhost:27017/",
        "cesfo_db",
        "balances",
    ).await?;
    println!("Connected to MongoDB successfully.");

    let creds = Credentials {
        badge_div: "6891".to_string(),
        badge_number: "18147".to_string(),
        badge_nom: "guillerot lucas".to_string(),
    };

    let client = CesfoClient::new()?;
    let html_body = client.authenticate(&creds).await?;

    let account_info = CesfoClient::parse_account_info(&html_body)?;

    db_service.save_account_info(account_info).await?;

    Ok(())
}
