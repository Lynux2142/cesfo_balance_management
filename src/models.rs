use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub balance: f64,
    pub balance_date: String,
    pub paid_payment: f64,
    pub pending_payment: f64,
    pub meal_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub badge_div: String,
    pub badge_number: String,
    pub badge_nom: String,
}
