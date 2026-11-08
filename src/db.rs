use mongodb::{Client, Collection, options::UpdateOptions};
use mongodb::bson::doc;
use crate::models::AccountInfo;

pub struct DatabaseService {
    collection: Collection<AccountInfo>,
}

impl DatabaseService {
    pub async fn new(uri: &str, db_name: &str, coll_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::with_uri_str(uri).await?;
        let database = client.database(db_name);
        let collection = database.collection::<AccountInfo>(coll_name);

        Ok(Self { collection })
    }

    pub async fn get_latest_account_info(&self) -> Result<Option<AccountInfo>, Box<dyn std::error::Error>> {
        let options = mongodb::options::FindOneOptions::builder()
            .sort(doc! { "balance_date": -1 })
            .build();

        let latest = self.collection.find_one(doc! {}).with_options(options).await?;
        Ok(latest)
    }

    pub async fn save_account_info(&self, mut info: AccountInfo) -> Result<(), Box<dyn std::error::Error>> {
        println!("Saving account info into MongoDB...");
        let previous_record = self.get_latest_account_info().await?;

        let (previous_balance, previous_pending) = match &previous_record {
            Some(last) => (last.balance, last.pending_payment),
            None => (info.balance, 0.0),
        };

        let adjusted_current_balance = info.balance - previous_pending;

        let raw_diff = adjusted_current_balance - previous_balance;

        let meal_price = (raw_diff.abs() * 100.0).round() / 100.0;
        info.meal_price = meal_price;

        let filter = doc! { "balance_date": &info.balance_date };
        
        let update = doc! {
            "$set": {
                "balance": info.balance,
                "paid_payment": info.paid_payment,
                "pending_payment": info.pending_payment,
                "meal_price": info.meal_price
            }
        };

        let options = UpdateOptions::builder().upsert(true).build();

        self.collection.update_one(filter, update).with_options(options).await?;
        
        println!("Successfully saved account info and synchronized database!");

        Ok(())
    }

    pub async fn get_latest_n_account_infos(&self, limit: i64) -> Result<Vec<AccountInfo>, Box<dyn std::error::Error>> {
        use futures::stream::StreamExt;
        
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "balance_date": -1 })
            .limit(limit)
            .build();

        let mut cursor = self.collection.find(doc! {}).with_options(options).await?;
        let mut records = Vec::new();

        while let Some(result) = cursor.next().await {
            if let Ok(record) = result {
                records.push(record);
            }
        }

        Ok(records)
    }
}
