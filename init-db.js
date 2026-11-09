db = db.getSiblingDB('cesfo_db');

db.createCollection("accounts");
db.createCollection('balances');

db.accounts.createIndex({ "badge_number": 1}, { unique: true });
db.balances.createIndex({ "badge_number": 1, "balance_date": 1}, { unique: true });
