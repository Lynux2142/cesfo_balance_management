use mongodb::{
    Client, Collection,
    options::{UpdateOptions},
    bson::doc,
};
use crate::models::{AccountInfo, AccountProfile};

#[derive(Clone)]
pub struct DatabaseService {
    accounts_collection: Collection<AccountProfile>,
    balances_collection: Collection<AccountInfo>,
}

impl DatabaseService {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::with_uri_str(uri).await?;
        let database = client.database(db_name);
        
        let accounts_collection = database.collection::<AccountProfile>("accounts");
        let balances_collection = database.collection::<AccountInfo>("balances");

        Ok(Self { accounts_collection, balances_collection })
    }

    pub async fn create_account(&self, profile: AccountProfile) -> Result<(), Box<dyn std::error::Error>> {
        self.accounts_collection.insert_one(profile).await?;
        Ok(())
    }

    pub async fn get_account_by_name(&self, first_name: &str, last_name: &str) -> Result<Option<AccountProfile>, Box<dyn std::error::Error>> {
        let filter = doc! { 
            "first_name": { "$regex": format!("^{}$", first_name), "$options": "i" },
            "last_name": { "$regex": format!("^{}$", last_name), "$options": "i" }
        };
        let account = self.accounts_collection.find_one(filter).await?;
        Ok(account)
    }

    pub async fn get_account_by_badge(&self, badge_number: &str) -> Result<Option<AccountProfile>, Box<dyn std::error::Error>> {
        let filter = doc! { "badge_number": badge_number };
        let account = self.accounts_collection.find_one(filter).await?;
        Ok(account)
    }

    pub async fn get_latest_account_info(&self, badge_number: &str) -> Result<Option<AccountInfo>, Box<dyn std::error::Error>> {
        let options = mongodb::options::FindOneOptions::builder()
            .sort(doc! { "balance_date": -1 })
            .build();

        let latest = self.balances_collection
            .find_one(doc! { "badge_number": badge_number })
            .with_options(options)
            .await?;
        Ok(latest)
    }

    pub async fn get_all_accounts(&self) -> Result<Vec<AccountProfile>, Box<dyn std::error::Error>> {
        use futures::stream::StreamExt;
        let mut cursor = self.accounts_collection.find(doc! {}).await?;
        let mut accounts = Vec::new();
        while let Some(result) = cursor.next().await {
            if let Ok(account) = result {
                accounts.push(account);
            }
        }
        Ok(accounts)
    }

    pub async fn get_latest_n_account_infos(&self, badge_number: &str, limit: i64) -> Result<Vec<AccountInfo>, Box<dyn std::error::Error>> {
        use futures::stream::StreamExt;
        
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "balance_date": -1 })
            .limit(limit)
            .build();

        let mut cursor = self.balances_collection
            .find(doc! { "badge_number": badge_number })
            .with_options(options)
            .await?;
            
        let mut records = Vec::new();
        while let Some(result) = cursor.next().await {
            if let Ok(record) = result {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub async fn save_account_info(&self, mut info: AccountInfo) -> Result<(), Box<dyn std::error::Error>> {
        println!("Saving account info into MongoDB...");
        let previous_record = self.get_latest_account_info(&info.badge_number).await?;

        let (previous_balance, previous_pending) = match &previous_record {
            Some(last) => (last.balance, last.pending_payment),
            None => (info.balance, 0.0),
        };

        let adjusted_current_balance = info.balance - previous_pending;
        let raw_diff = adjusted_current_balance - previous_balance;
        info.meal_price = (raw_diff.abs() * 100.0).round() / 100.0;

        let filter = doc! { 
            "badge_number": &info.badge_number,
            "balance_date": &info.balance_date 
        };
        
        let update = doc! {
            "$set": {
                "balance": info.balance,
                "paid_payment": info.paid_payment,
                "pending_payment": info.pending_payment,
                "meal_price": info.meal_price
            }
        };

        let options = UpdateOptions::builder().upsert(true).build();
        self.balances_collection.update_one(filter, update).with_options(options).await?;
        
        println!("Successfully saved account info and synchronized database!");
        Ok(())
    }
}
