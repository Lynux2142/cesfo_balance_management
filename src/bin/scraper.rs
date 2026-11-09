use cesfo_balance_management::{CesfoClient, DatabaseService, Credentials};
use std::env;
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mongo_url = env::var("MONGODB_URL").expect("MANDATORY ERROR: MONGODB_URL is not set.");
    let mongo_port = env::var("MONGODB_PORT").expect("MANDATORY ERROR: MONGODB_PORT is not set.");
    let mongo_user = env::var("MONGODB_USER").expect("MANDATORY ERROR: MONGODB_USER is not set.");
    let mongo_password = env::var("MONGODB_PASSWORD").expect("MANDATORY ERROR: MONGODB_PASSWORD is not set.");

    let mongo_uri = format!("mongodb://{}:{}@{}:{}", mongo_user, mongo_password, mongo_url, mongo_port);
    let db_service = DatabaseService::new(&mongo_uri, "cesfo_db").await?;

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let badge_number = &args[1];
        println!("Starting manual CESFO scrape for badge: {}...", badge_number);

        let account_profile = match db_service.get_account_by_badge(badge_number).await? {
            Some(profile) => profile,
            None => {
                eprintln!("Error: No account found with badge number '{}' in the database.", badge_number);
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
        println!("Manual scrape completed and saved successfully.");
        return Ok(());
    }

    let cron_schedule = env::var("CRON_SCHEDULE").expect("MANDATORY ERROR: CRON_SCHEDULE is not set.");
    println!("Starting CESFO Cron Scraper Worker with schedule: '{}'...", cron_schedule);

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
                    println!("[CRON] Scraping for account: {} {} (Badge: {})", acc.first_name, acc.last_name, acc.badge_number);
                    
                    let creds = Credentials {
                        badge_div: acc.badge_div,
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
    println!("[SCHEDULER] Cron worker is running and waiting for schedule triggers...");

    tokio::signal::ctrl_c().await?;
    println!("Shutting down cron worker...");

    Ok(())
}
