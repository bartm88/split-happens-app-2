pub mod sheets_dao;
pub mod memory_dao;
pub mod dynamodb_dao;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub creditor: String,
    pub debtor: String,
    pub amount: f64,
    pub split: String,
    pub time: String,
    pub pot_amount: f64,
    pub date: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Balance {
    pub name: String,
    pub amount: String,
}

#[async_trait]
pub trait StorageDao: Send + Sync {
    async fn get_names(&self) -> Vec<String>;
    async fn get_balances(&self) -> Vec<Balance>;
    async fn get_last_n_transactions(&self, n: usize) -> Vec<Transaction>;
    async fn remove_last_transaction(&self);
    async fn add_split(&self, name: String, split: String);
    async fn add_conversion(&self, name: String, split: String);
    async fn get_split_awards(&self) -> HashMap<String, f64>;
}