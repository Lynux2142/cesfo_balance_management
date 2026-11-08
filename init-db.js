db = db.getSiblingDB('cesfo_db');

db.createCollection('balances');

db.balances.createIndex({ "balance_date": 1}, { unique: true });
