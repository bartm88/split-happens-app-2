use super::{Balance, StorageDao, Transaction};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct MemoryDao {
    transactions: Arc<Mutex<Vec<Transaction>>>,
    split_awards: Arc<Mutex<HashMap<String, f64>>>,
    names: Arc<Mutex<Vec<String>>>,
}

impl MemoryDao {
    pub fn new() -> Self {
        let mut dao = Self {
            transactions: Arc::new(Mutex::new(Vec::new())),
            split_awards: Arc::new(Mutex::new(HashMap::new())),
            names: Arc::new(Mutex::new(Vec::new())),
        };
        
        // Initialize with test data
        dao.init_test_data();
        dao
    }
    
    fn init_test_data(&mut self) {
        // Initialize names
        {
            let mut names = self.names.lock().unwrap();
            names.extend_from_slice(&[
                "Alice".to_string(),
                "Bob".to_string(), 
                "Charlie".to_string(),
                "Dana".to_string(),
                "Pot".to_string(),
            ]);
        }
        
        // Initialize split awards
        {
            let mut awards = self.split_awards.lock().unwrap();
            awards.insert("1-2-3".to_string(), 5.0);
            awards.insert("2-3".to_string(), 7.5);
            awards.insert("3-6".to_string(), 10.0);
            awards.insert("4-5".to_string(), 12.5);
            awards.insert("4-5-6".to_string(), 15.0);
            awards.insert("5-6".to_string(), 17.5);
            awards.insert("4-7-10".to_string(), 20.0);
            awards.insert("6-7-10".to_string(), 22.5);
            awards.insert("7-10".to_string(), 25.0);
        }
        
        // Add some test transactions
        {
            let mut transactions = self.transactions.lock().unwrap();
            
            // Add some initial splits
            transactions.push(Transaction {
                creditor: "Alice".to_string(),
                debtor: "Pot".to_string(),
                amount: 1.0,
                split: "7-10".to_string(),
                time: "1/15/2025, 2:30:00 PM UTC".to_string(),
                pot_amount: 10.0,
                date: "1/15/2025".to_string(),
            });
            
            transactions.push(Transaction {
                creditor: "Bob".to_string(),
                debtor: "Pot".to_string(),
                amount: 1.0,
                split: "4-5".to_string(),
                time: "1/15/2025, 2:45:00 PM UTC".to_string(),
                pot_amount: 11.0,
                date: "1/15/2025".to_string(),
            });
            
            transactions.push(Transaction {
                creditor: "Charlie".to_string(),
                debtor: "Pot".to_string(),
                amount: 1.0,
                split: "2-3".to_string(),
                time: "1/15/2025, 3:00:00 PM UTC".to_string(),
                pot_amount: 12.0,
                date: "1/15/2025".to_string(),
            });
            
            // Add a conversion
            transactions.push(Transaction {
                creditor: "Pot".to_string(),
                debtor: "Alice".to_string(),
                amount: 3.0,  // 25% of 12.0
                split: "7-10".to_string(),
                time: "1/15/2025, 3:15:00 PM UTC".to_string(),
                pot_amount: 12.0,
                date: "1/15/2025".to_string(),
            });
        }
    }
    
    fn calculate_balances(&self) -> Vec<Balance> {
        let transactions = self.transactions.lock().unwrap();
        let mut balances: HashMap<String, f64> = HashMap::new();
        
        // Initialize all names with 0 balance
        for name in self.names.lock().unwrap().iter() {
            balances.insert(name.clone(), 0.0);
        }
        
        // Calculate balances from transactions
        for transaction in transactions.iter() {
            // Creditor gets positive amount
            *balances.entry(transaction.creditor.clone()).or_insert(0.0) += transaction.amount;
            // Debtor gets negative amount
            *balances.entry(transaction.debtor.clone()).or_insert(0.0) -= transaction.amount;
        }
        
        // Convert to Balance structs
        balances.into_iter()
            .map(|(name, amount)| Balance {
                name,
                amount: format!("{:.2}", amount),
            })
            .collect()
    }
}

#[async_trait]
impl StorageDao for MemoryDao {
    async fn get_names(&self) -> Vec<String> {
        self.names.lock().unwrap().clone()
    }
    
    async fn get_balances(&self) -> Vec<Balance> {
        self.calculate_balances()
    }
    
    async fn get_last_n_transactions(&self, n: usize) -> Vec<Transaction> {
        let transactions = self.transactions.lock().unwrap();
        let start = transactions.len().saturating_sub(n);
        transactions[start..].to_vec()
    }
    
    async fn remove_last_transaction(&self) {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.pop();
    }
    
    async fn add_split(&self, name: String, split: String) {
        let pot_balance = self.calculate_balances()
            .iter()
            .find(|b| b.name == "Pot")
            .map(|b| b.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
            
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();
        
        let transaction = Transaction {
            creditor: name,
            debtor: "Pot".to_string(),
            amount: 1.0,
            split,
            time: now_string,
            pot_amount: pot_balance,
            date: today_string,
        };
        
        let mut transactions = self.transactions.lock().unwrap();
        transactions.push(transaction);
    }
    
    async fn add_conversion(&self, name: String, split: String) {
        let pot_balance = self.calculate_balances()
            .iter()
            .find(|b| b.name == "Pot")
            .map(|b| b.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
            
        let split_awards = self.split_awards.lock().unwrap();
        let award_multiplier_percent = split_awards
            .get(&split)
            .expect(&format!("Invalid split: {}", split));
        let award = (pot_balance * award_multiplier_percent).round() / 100.0;
        
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();
        
        let transaction = Transaction {
            creditor: "Pot".to_string(),
            debtor: name,
            amount: award,
            split,
            time: now_string,
            pot_amount: pot_balance,
            date: today_string,
        };
        
        let mut transactions = self.transactions.lock().unwrap();
        transactions.push(transaction);
    }
    
    async fn get_split_awards(&self) -> HashMap<String, f64> {
        self.split_awards.lock().unwrap().clone()
    }
}