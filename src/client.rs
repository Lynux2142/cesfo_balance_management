use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use chrono::NaiveDateTime;
use crate::models::{AccountInfo, Credentials};

pub struct CesfoClient {
    client: Client,
    base_url: String,
}

fn parse_currency_to_float(s: &str) -> f64 {
    let cleaned = s.replace('€', "").replace('\u{a0}', "").trim().to_string();
    let normalized = cleaned.replace(',', ".");
    
    match normalized.parse::<f64>() {
        Ok(val) => val,
        Err(_) => 0.0,
    }
}

impl CesfoClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .cookie_store(true)
            .build()?;

        Ok(Self {
            client,
            base_url: "https://www.e-chargement.com".to_string(),
        })
    }

    pub async fn authenticate(&self, creds: &Credentials) -> Result<String, Box<dyn std::error::Error>> {
        println!("Authenticating ...");

        let _ = self.client.get(&format!("{}/default.asp", self.base_url))
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:159.0) Gecko/20100101 Firefox/159.0")
            .send()
            .await?;

        let mut params = HashMap::new();
        params.insert("badge_div", creds.badge_div.clone());
        params.insert("badge_number", creds.badge_number.clone());
        params.insert("badge_nom", creds.badge_nom.clone());

        let login_url = format!("{}/identif_badge.Asp", self.base_url);

        let response = self.client.post(&login_url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:159.0) Gecko/20100101 Firefox/159.0")
            .header("Referer", format!("{}/default.asp", self.base_url))
            .header("Origin", self.base_url.clone())
            .form(&params)
            .send()
            .await?;

        let html_body = response.text().await?;
        println!("Successfully authenticated. HTML response received.");

        Ok(html_body)
    }

    pub fn parse_account_info(html_content: &str) -> Result<AccountInfo, Box<dyn std::error::Error>> {
        println!("Parsing account info ...");
        let document = Html::parse_document(html_content);
        
        let mut raw_balance = String::from("N/A");
        let mut raw_balance_date = String::from("N/A");
        let mut raw_paid_payment = String::from("N/A");
        let mut raw_pending_payment = String::from("N/A");

        let compte_selector = Selector::parse("div#compte").unwrap();
        
        if let Some(compte_element) = document.select(&compte_selector).next() {
            let row_selector = Selector::parse("table tr").unwrap();
            let bold_selector = Selector::parse("td.bold").unwrap();

            let compte_html = compte_element.html();
            let compte_fragment = Html::parse_fragment(&compte_html);

            for row in compte_fragment.select(&row_selector) {
                let row_text = row.text().collect::<String>();

                if row_text.contains("Solde") && !row_text.contains("Date") && !row_text.contains("Crediter") {
                    if let Some(bold_element) = row.select(&bold_selector).next() {
                        raw_balance = bold_element.inner_html().trim().to_string();
                    }
                }

                if row_text.contains("Date du solde") {
                    if let Some(bold_element) = row.select(&bold_selector).next() {
                        raw_balance_date = bold_element.inner_html().trim().to_string();
                    }
                }

                if row_text.contains("Paiement credite depuis cette date") {
                    if let Some(bold_element) = row.select(&bold_selector).next() {
                        raw_paid_payment = bold_element.inner_html().trim().to_string();
                    }
                }

                if row_text.contains("Paiement en cours depuis cette date") {
                    if let Some(bold_element) = row.select(&bold_selector).next() {
                        raw_pending_payment = bold_element.inner_html().trim().to_string();
                    }
                }
            }
        }

        let balance = parse_currency_to_float(&raw_balance);
        let paid_payment = parse_currency_to_float(&raw_paid_payment);
        let pending_payment = parse_currency_to_float(&raw_pending_payment);
        let balance_date = match NaiveDateTime::parse_from_str(&raw_balance_date, "%d/%m/%y à %Hh%M") {
            Ok(dt) => dt.format("%Y-%m-%d").to_string(),
            Err(_) => raw_balance_date,
        };

        println!("Parsed account info:");
        println!("Balance           : {}", raw_balance);
        println!("Balance Date      : {}", balance_date);
        println!("Paid Payment      : {}", raw_paid_payment);
        println!("Pending Payment   : {}", raw_pending_payment);

        Ok(AccountInfo {
            balance,
            balance_date,
            paid_payment,
            pending_payment,
            meal_price: 0.0,
        })
    }
}
