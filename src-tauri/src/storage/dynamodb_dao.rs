use super::{Balance, StorageDao, Transaction};
use crate::secrets::{AWS_ACCESS_KEY_ID, AWS_REGION, AWS_SECRET_ACCESS_KEY};
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::SystemTime;

pub struct DynamoDbDao {
    client: Client,
    games_table: String,
    transactions_table: String,
    game_id: String,
}

impl DynamoDbDao {
    pub async fn new() -> Self {
        // Create AWS config with credentials from secrets
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(AWS_REGION))
            .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
                AWS_ACCESS_KEY_ID,
                AWS_SECRET_ACCESS_KEY,
                None, // session token
                None, // expiration
                "split-happens-static-credentials",
            ))
            .load()
            .await;

        let client = Client::new(&config);

        Self {
            client,
            games_table: "split-happens-games".to_string(),
            transactions_table: "split-happens-transactions".to_string(),
            game_id: "sample_game_id".to_string(),
        }
    }

    async fn get_game(
        &self,
    ) -> Result<Option<HashMap<String, AttributeValue>>, aws_sdk_dynamodb::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.games_table)
            .key("game_id", AttributeValue::S(self.game_id.clone()))
            .send()
            .await?;

        Ok(response.item)
    }

    async fn get_next_transaction_number(&self) -> Result<i32, aws_sdk_dynamodb::Error> {
        match self.get_game().await? {
            Some(game_item) => {
                if let Some(AttributeValue::N(next_num_str)) = game_item.get("next_transaction_num")
                {
                    if let Ok(next_num) = next_num_str.parse::<i32>() {
                        return Ok(next_num);
                    }
                }
                // If no next_transaction_num field, default to 1
                Ok(1)
            }
            None => {
                // If no game record exists, start with 1
                Ok(1)
            }
        }
    }

    async fn put_transaction_with_lock(
        &self,
        transaction: &Transaction,
        transaction_number: i32,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        let mut item = HashMap::new();
        item.insert(
            "game_id".to_string(),
            AttributeValue::S(self.game_id.clone()),
        );
        item.insert(
            "transaction_num".to_string(),
            AttributeValue::N(transaction_number.to_string()),
        );
        item.insert(
            "creditor".to_string(),
            AttributeValue::S(transaction.creditor.clone()),
        );
        item.insert(
            "debtor".to_string(),
            AttributeValue::S(transaction.debtor.clone()),
        );
        item.insert(
            "amount".to_string(),
            AttributeValue::N(transaction.amount.to_string()),
        );
        item.insert(
            "split".to_string(),
            AttributeValue::S(transaction.split.clone()),
        );
        item.insert(
            "time".to_string(),
            AttributeValue::S(transaction.time.clone()),
        );
        item.insert(
            "pot_amount".to_string(),
            AttributeValue::N(transaction.pot_amount.to_string()),
        );
        item.insert(
            "date".to_string(),
            AttributeValue::S(transaction.date.clone()),
        );

        // Optimistic lock - only put if transaction_num doesn't exist
        self.client
            .put_item()
            .table_name(&self.transactions_table)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(transaction_num)")
            .send()
            .await?;

        Ok(())
    }

    async fn update_game_with_transaction(
        &self,
        transaction: &Transaction,
        transaction_number: i32,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        // Get current game state to update balances incrementally
        let current_balances = match self.get_game().await? {
            Some(game_item) => {
                if let Some(AttributeValue::M(balances_map)) = game_item.get("balances") {
                    balances_map
                        .iter()
                        .filter_map(|(name, value)| {
                            if let Ok(amount_str) = value.as_n() {
                                if let Ok(amount) = amount_str.parse::<f64>() {
                                    return Some((name.clone(), amount));
                                }
                            }
                            None
                        })
                        .collect::<HashMap<String, f64>>()
                } else {
                    HashMap::new()
                }
            }
            None => HashMap::new(),
        };

        // Calculate new balances based on the transaction
        let mut new_balances = current_balances;
        // Creditor gets positive amount
        *new_balances
            .entry(transaction.creditor.clone())
            .or_insert(0.0) += transaction.amount;
        // Debtor gets negative amount
        *new_balances
            .entry(transaction.debtor.clone())
            .or_insert(0.0) -= transaction.amount;

        // Convert to DynamoDB format
        let balance_map: HashMap<String, AttributeValue> = new_balances
            .into_iter()
            .map(|(name, amount)| (name, AttributeValue::N(amount.to_string())))
            .collect();

        // Update both balances and next_transaction_num atomically
        self.client
            .update_item()
            .table_name(&self.games_table)
            .key("game_id", AttributeValue::S(self.game_id.clone()))
            .update_expression("SET balances = :balances, next_transaction_num = :next_num")
            .expression_attribute_values(":balances", AttributeValue::M(balance_map))
            .expression_attribute_values(
                ":next_num",
                AttributeValue::N((transaction_number + 1).to_string()),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn reverse_transaction_in_game(
        &self,
        transaction: &Transaction,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        // Get current game state
        let current_balances = match self.get_game().await? {
            Some(game_item) => {
                if let Some(AttributeValue::M(balances_map)) = game_item.get("balances") {
                    balances_map
                        .iter()
                        .filter_map(|(name, value)| {
                            if let Ok(amount_str) = value.as_n() {
                                if let Ok(amount) = amount_str.parse::<f64>() {
                                    return Some((name.clone(), amount));
                                }
                            }
                            None
                        })
                        .collect::<HashMap<String, f64>>()
                } else {
                    HashMap::new()
                }
            }
            None => HashMap::new(),
        };

        // Reverse the transaction effects
        let mut new_balances = current_balances;
        // Reverse creditor's positive amount
        *new_balances
            .entry(transaction.creditor.clone())
            .or_insert(0.0) -= transaction.amount;
        // Reverse debtor's negative amount
        *new_balances
            .entry(transaction.debtor.clone())
            .or_insert(0.0) += transaction.amount;

        // Convert to DynamoDB format
        let balance_map: HashMap<String, AttributeValue> = new_balances
            .into_iter()
            .map(|(name, amount)| (name, AttributeValue::N(amount.to_string())))
            .collect();

        // Update balances
        self.client
            .update_item()
            .table_name(&self.games_table)
            .key("game_id", AttributeValue::S(self.game_id.clone()))
            .update_expression("SET balances = :balances")
            .expression_attribute_values(":balances", AttributeValue::M(balance_map))
            .send()
            .await?;

        Ok(())
    }

    async fn delete_transaction(
        &self,
        transaction_number: i32,
    ) -> Result<(), aws_sdk_dynamodb::Error> {
        self.client
            .delete_item()
            .table_name(&self.transactions_table)
            .key("game_id", AttributeValue::S(self.game_id.clone()))
            .key(
                "transaction_num",
                AttributeValue::N(transaction_number.to_string()),
            )
            .send()
            .await?;

        Ok(())
    }

    fn item_to_transaction(&self, item: &HashMap<String, AttributeValue>) -> Option<Transaction> {
        Some(Transaction {
            creditor: item.get("creditor")?.as_s().ok()?.clone(),
            debtor: item.get("debtor")?.as_s().ok()?.clone(),
            amount: item.get("amount")?.as_n().ok()?.parse().ok()?,
            split: item.get("split")?.as_s().ok()?.clone(),
            time: item.get("time")?.as_s().ok()?.clone(),
            pot_amount: item.get("pot_amount")?.as_n().ok()?.parse().ok()?,
            date: item.get("date")?.as_s().ok()?.clone(),
        })
    }

    async fn get_balances_from_game(&self) -> Vec<Balance> {
        match self.get_game().await {
            Ok(Some(game_item)) => {
                if let Some(AttributeValue::M(balances_map)) = game_item.get("balances") {
                    balances_map
                        .iter()
                        .filter_map(|(name, value)| {
                            if let Ok(amount_str) = value.as_n() {
                                if let Ok(amount) = amount_str.parse::<f64>() {
                                    return Some(Balance {
                                        name: name.clone(),
                                        amount: format!("{:.2}", amount),
                                    });
                                }
                            }
                            None
                        })
                        .collect()
                } else {
                    panic!("No balances in game");
                }
            }
            _ => {
                panic!("No game found");
            }
        }
    }

    async fn get_last_transaction_and_number(&self) -> (Transaction, i32) {
        match self
            .client
            .query()
            .table_name(&self.transactions_table)
            .key_condition_expression("game_id = :game_id")
            .expression_attribute_values(":game_id", AttributeValue::S(self.game_id.clone()))
            .scan_index_forward(false)
            .limit(1 as i32) // Limit to n items
            .send()
            .await
        {
            Ok(response) => response
                .items
                .unwrap_or_default()
                .iter()
                .map(|item| {
                    let transaction_num = item
                        .get("transaction_num")
                        .unwrap()
                        .as_n()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let transaction = self.item_to_transaction(&item).unwrap();
                    (transaction, transaction_num)
                })
                .next()
                .unwrap(),
            Err(e) => {
                log::error!("Failed to get last n transactions: {:?}", e);
                panic!("Failed to get last n transactions: {:?}", e);
            }
        }
    }
}

#[async_trait]
impl StorageDao for DynamoDbDao {
    async fn get_names(&self) -> Vec<String> {
        match self.get_game().await {
            Ok(Some(game_item)) => {
                if let Some(AttributeValue::Ss(players)) = game_item.get("players") {
                    let mut names = players.clone();
                    // Always ensure "Pot" is included
                    if !names.contains(&"Pot".to_string()) {
                        names.push("Pot".to_string());
                    }
                    names
                } else {
                    panic!("No players in game");
                }
            }
            _ => {
                panic!("No game found");
            }
        }
    }

    async fn get_balances(&self) -> Vec<Balance> {
        self.get_balances_from_game().await
    }

    async fn get_last_n_transactions(&self, n: usize) -> Vec<Transaction> {
        // Use DynamoDB query with descending sort order and limit
        match self
            .client
            .query()
            .table_name(&self.transactions_table)
            .key_condition_expression("game_id = :game_id")
            .expression_attribute_values(":game_id", AttributeValue::S(self.game_id.clone()))
            .scan_index_forward(true) // Sort by sort key (transaction_num) in descending order
            .limit(n as i32) // Limit to n items
            .send()
            .await
        {
            Ok(response) => response
                .items
                .unwrap_or_default()
                .iter()
                .filter_map(|item| self.item_to_transaction(item))
                .collect(),
            Err(e) => {
                log::error!("Failed to get last n transactions: {:?}", e);
                Vec::new()
            }
        }
    }

    async fn remove_last_transaction(&self) {
        let (last_transaction, last_transaction_num) = self.get_last_transaction_and_number().await;

        if let Err(e) = self.delete_transaction(last_transaction_num).await {
            log::error!("Failed to delete transaction: {:?}", e);
            return;
        }

        // Reverse the transaction effects in the game balances
        if let Err(e) = self.reverse_transaction_in_game(&last_transaction).await {
            log::error!("Failed to reverse transaction in game: {:?}", e);
        }
    }

    async fn add_split(&self, name: String, split: String) {
        let pot_balance = self
            .get_balances_from_game()
            .await
            .iter()
            .find(|b| b.name == "Pot")
            .map(|b| b.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();

        let transaction = Transaction {
            creditor: "Pot".to_string(),
            debtor: name,
            amount: 1.0,
            split,
            time: now_string.clone(),
            pot_amount: pot_balance,
            date: today_string,
        };

        // Get next transaction number and try to add with optimistic lock
        match self.get_next_transaction_number().await {
            Ok(transaction_number) => {
                if let Err(e) = self
                    .put_transaction_with_lock(&transaction, transaction_number)
                    .await
                {
                    log::error!("Failed to add split transaction: {:?}", e);
                    return;
                }

                // Update game balances and next_transaction_num atomically
                if let Err(e) = self
                    .update_game_with_transaction(&transaction, transaction_number)
                    .await
                {
                    log::error!("Failed to update game with transaction: {:?}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to get next transaction number: {:?}", e);
            }
        }
    }

    async fn add_conversion(&self, name: String, split: String) {
        let pot_balance = self
            .get_balances_from_game()
            .await
            .iter()
            .find(|b| b.name == "Pot")
            .map(|b| b.amount.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        let split_awards = self.get_split_awards().await;
        let award_multiplier_percent = split_awards
            .get(&split)
            .expect(&format!("Invalid split: {}", split));
        let award = (pot_balance * award_multiplier_percent).round() / 100.0;

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();

        let transaction = Transaction {
            creditor: name,
            debtor: "Pot".to_string(),
            amount: award,
            split,
            time: now_string.clone(),
            pot_amount: pot_balance,
            date: today_string,
        };

        // Get next transaction number and try to add with optimistic lock
        match self.get_next_transaction_number().await {
            Ok(transaction_number) => {
                if let Err(e) = self
                    .put_transaction_with_lock(&transaction, transaction_number)
                    .await
                {
                    log::error!("Failed to add conversion transaction: {:?}", e);
                    return;
                }

                // Update game balances and next_transaction_num atomically
                if let Err(e) = self
                    .update_game_with_transaction(&transaction, transaction_number)
                    .await
                {
                    log::error!("Failed to update game with transaction: {:?}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to get next transaction number: {:?}", e);
            }
        }
    }

    async fn get_split_awards(&self) -> HashMap<String, f64> {
        let mut awards = HashMap::new();

        // 10% splits
        awards.insert("2-3".to_string(), 10.0);
        awards.insert("2-7".to_string(), 10.0);
        awards.insert("2-9".to_string(), 10.0);
        awards.insert("3-10".to_string(), 10.0);
        awards.insert("3-8".to_string(), 10.0);
        awards.insert("4-5".to_string(), 10.0);
        awards.insert("5-6".to_string(), 10.0);
        awards.insert("7-8".to_string(), 10.0);
        awards.insert("8-9".to_string(), 10.0);
        awards.insert("9-10".to_string(), 10.0);

        // 30% splits
        awards.insert("2-10".to_string(), 30.0);
        awards.insert("2-3-10".to_string(), 30.0);
        awards.insert("2-3-4".to_string(), 30.0);
        awards.insert("2-3-4-10".to_string(), 30.0);
        awards.insert("2-3-4-5".to_string(), 30.0);
        awards.insert("2-3-4-5-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6".to_string(), 30.0);
        awards.insert("2-3-4-5-6-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-8".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-8-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-8-9".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-9".to_string(), 30.0);
        awards.insert("2-3-4-5-6-7-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-8".to_string(), 30.0);
        awards.insert("2-3-4-5-6-8-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-8-9".to_string(), 30.0);
        awards.insert("2-3-4-5-6-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-6-9".to_string(), 30.0);
        awards.insert("2-3-4-5-6-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-7".to_string(), 30.0);
        awards.insert("2-3-4-5-7-10".to_string(), 30.0);
        awards.insert("2-3-4-5-7-8".to_string(), 30.0);
        awards.insert("2-3-4-5-7-8-10".to_string(), 30.0);
        awards.insert("2-3-4-5-7-8-9".to_string(), 30.0);
        awards.insert("2-3-4-5-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-7-9".to_string(), 30.0);
        awards.insert("2-3-4-5-7-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-8".to_string(), 30.0);
        awards.insert("2-3-4-5-8-10".to_string(), 30.0);
        awards.insert("2-3-4-5-8-9".to_string(), 30.0);
        awards.insert("2-3-4-5-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-5-9".to_string(), 30.0);
        awards.insert("2-3-4-5-9-10".to_string(), 30.0);
        awards.insert("2-3-4-6".to_string(), 30.0);
        awards.insert("2-3-4-6-10".to_string(), 30.0);
        awards.insert("2-3-4-6-7".to_string(), 30.0);
        awards.insert("2-3-4-6-7-10".to_string(), 30.0);
        awards.insert("2-3-4-6-7-8".to_string(), 30.0);
        awards.insert("2-3-4-6-7-8-10".to_string(), 30.0);
        awards.insert("2-3-4-6-7-8-9".to_string(), 30.0);
        awards.insert("2-3-4-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-6-7-9".to_string(), 30.0);
        awards.insert("2-3-4-6-7-9-10".to_string(), 30.0);
        awards.insert("2-3-4-6-8".to_string(), 30.0);
        awards.insert("2-3-4-6-8-10".to_string(), 30.0);
        awards.insert("2-3-4-6-8-9".to_string(), 30.0);
        awards.insert("2-3-4-6-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-6-9".to_string(), 30.0);
        awards.insert("2-3-4-6-9-10".to_string(), 30.0);
        awards.insert("2-3-4-7".to_string(), 30.0);
        awards.insert("2-3-4-7-10".to_string(), 30.0);
        awards.insert("2-3-4-7-8".to_string(), 30.0);
        awards.insert("2-3-4-7-8-10".to_string(), 30.0);
        awards.insert("2-3-4-7-8-9".to_string(), 30.0);
        awards.insert("2-3-4-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-7-9".to_string(), 30.0);
        awards.insert("2-3-4-7-9-10".to_string(), 30.0);
        awards.insert("2-3-4-8".to_string(), 30.0);
        awards.insert("2-3-4-8-10".to_string(), 30.0);
        awards.insert("2-3-4-8-9".to_string(), 30.0);
        awards.insert("2-3-4-8-9-10".to_string(), 30.0);
        awards.insert("2-3-4-9".to_string(), 30.0);
        awards.insert("2-3-4-9-10".to_string(), 30.0);
        awards.insert("2-3-5".to_string(), 30.0);
        awards.insert("2-3-5-10".to_string(), 30.0);
        awards.insert("2-3-5-6".to_string(), 30.0);
        awards.insert("2-3-5-6-10".to_string(), 30.0);
        awards.insert("2-3-5-6-7".to_string(), 30.0);
        awards.insert("2-3-5-6-7-10".to_string(), 30.0);
        awards.insert("2-3-5-6-7-8".to_string(), 30.0);
        awards.insert("2-3-5-6-7-8-10".to_string(), 30.0);
        awards.insert("2-3-5-6-7-8-9".to_string(), 30.0);
        awards.insert("2-3-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-5-6-7-9".to_string(), 30.0);
        awards.insert("2-3-5-6-7-9-10".to_string(), 30.0);
        awards.insert("2-3-5-6-8".to_string(), 30.0);
        awards.insert("2-3-5-6-8-10".to_string(), 30.0);
        awards.insert("2-3-5-6-8-9".to_string(), 30.0);
        awards.insert("2-3-5-6-8-9-10".to_string(), 30.0);
        awards.insert("2-3-5-6-9".to_string(), 30.0);
        awards.insert("2-3-5-6-9-10".to_string(), 30.0);
        awards.insert("2-3-5-7".to_string(), 30.0);
        awards.insert("2-3-5-7-10".to_string(), 30.0);
        awards.insert("2-3-5-7-8".to_string(), 30.0);
        awards.insert("2-3-5-7-8-10".to_string(), 30.0);
        awards.insert("2-3-5-7-8-9".to_string(), 30.0);
        awards.insert("2-3-5-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-5-7-9".to_string(), 30.0);
        awards.insert("2-3-5-7-9-10".to_string(), 30.0);
        awards.insert("2-3-5-8".to_string(), 30.0);
        awards.insert("2-3-5-8-10".to_string(), 30.0);
        awards.insert("2-3-5-8-9".to_string(), 30.0);
        awards.insert("2-3-5-8-9-10".to_string(), 30.0);
        awards.insert("2-3-5-9".to_string(), 30.0);
        awards.insert("2-3-5-9-10".to_string(), 30.0);
        awards.insert("2-3-6".to_string(), 30.0);
        awards.insert("2-3-6-10".to_string(), 30.0);
        awards.insert("2-3-6-7".to_string(), 30.0);
        awards.insert("2-3-6-7-10".to_string(), 30.0);
        awards.insert("2-3-6-7-8".to_string(), 30.0);
        awards.insert("2-3-6-7-8-10".to_string(), 30.0);
        awards.insert("2-3-6-7-8-9".to_string(), 30.0);
        awards.insert("2-3-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-6-7-9".to_string(), 30.0);
        awards.insert("2-3-6-7-9-10".to_string(), 30.0);
        awards.insert("2-3-6-8".to_string(), 30.0);
        awards.insert("2-3-6-8-10".to_string(), 30.0);
        awards.insert("2-3-6-8-9".to_string(), 30.0);
        awards.insert("2-3-6-8-9-10".to_string(), 30.0);
        awards.insert("2-3-6-9".to_string(), 30.0);
        awards.insert("2-3-6-9-10".to_string(), 30.0);
        awards.insert("2-3-7".to_string(), 30.0);
        awards.insert("2-3-7-10".to_string(), 30.0);
        awards.insert("2-3-7-8".to_string(), 30.0);
        awards.insert("2-3-7-8-10".to_string(), 30.0);
        awards.insert("2-3-7-8-9".to_string(), 30.0);
        awards.insert("2-3-7-8-9-10".to_string(), 30.0);
        awards.insert("2-3-7-9".to_string(), 30.0);
        awards.insert("2-3-7-9-10".to_string(), 30.0);
        awards.insert("2-3-8".to_string(), 30.0);
        awards.insert("2-3-8-10".to_string(), 30.0);
        awards.insert("2-3-8-9".to_string(), 30.0);
        awards.insert("2-3-8-9-10".to_string(), 30.0);
        awards.insert("2-3-9".to_string(), 30.0);
        awards.insert("2-3-9-10".to_string(), 30.0);
        awards.insert("2-4-10".to_string(), 30.0);
        awards.insert("2-4-5-10".to_string(), 30.0);
        awards.insert("2-4-5-6".to_string(), 30.0);
        awards.insert("2-4-5-6-10".to_string(), 30.0);
        awards.insert("2-4-5-6-7".to_string(), 30.0);
        awards.insert("2-4-5-6-7-10".to_string(), 30.0);
        awards.insert("2-4-5-6-7-8".to_string(), 30.0);
        awards.insert("2-4-5-6-7-8-10".to_string(), 30.0);
        awards.insert("2-4-5-6-7-8-9".to_string(), 30.0);
        awards.insert("2-4-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-4-5-6-7-9".to_string(), 30.0);
        awards.insert("2-4-5-6-7-9-10".to_string(), 30.0);
        awards.insert("2-4-5-6-8".to_string(), 30.0);
        awards.insert("2-4-5-6-8-10".to_string(), 30.0);
        awards.insert("2-4-5-6-8-9".to_string(), 30.0);
        awards.insert("2-4-5-6-8-9-10".to_string(), 30.0);
        awards.insert("2-4-5-6-9".to_string(), 30.0);
        awards.insert("2-4-5-6-9-10".to_string(), 30.0);
        awards.insert("2-4-5-7-10".to_string(), 30.0);
        awards.insert("2-4-5-7-8-10".to_string(), 30.0);
        awards.insert("2-4-5-7-8-9-10".to_string(), 30.0);
        awards.insert("2-4-5-7-9-10".to_string(), 30.0);
        awards.insert("2-4-5-8-10".to_string(), 30.0);
        awards.insert("2-4-5-8-9-10".to_string(), 30.0);
        awards.insert("2-4-5-9-10".to_string(), 30.0);
        awards.insert("2-4-6".to_string(), 30.0);
        awards.insert("2-4-6-10".to_string(), 30.0);
        awards.insert("2-4-6-7".to_string(), 30.0);
        awards.insert("2-4-6-7-10".to_string(), 30.0);
        awards.insert("2-4-6-7-8".to_string(), 30.0);
        awards.insert("2-4-6-7-8-10".to_string(), 30.0);
        awards.insert("2-4-6-7-8-9".to_string(), 30.0);
        awards.insert("2-4-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-4-6-7-9".to_string(), 30.0);
        awards.insert("2-4-6-7-9-10".to_string(), 30.0);
        awards.insert("2-4-6-8".to_string(), 30.0);
        awards.insert("2-4-6-8-10".to_string(), 30.0);
        awards.insert("2-4-6-8-9".to_string(), 30.0);
        awards.insert("2-4-6-8-9-10".to_string(), 30.0);
        awards.insert("2-4-6-9".to_string(), 30.0);
        awards.insert("2-4-6-9-10".to_string(), 30.0);
        awards.insert("2-4-7-10".to_string(), 30.0);
        awards.insert("2-4-7-8-10".to_string(), 30.0);
        awards.insert("2-4-7-8-9".to_string(), 30.0);
        awards.insert("2-4-7-8-9-10".to_string(), 30.0);
        awards.insert("2-4-7-9".to_string(), 30.0);
        awards.insert("2-4-7-9-10".to_string(), 30.0);
        awards.insert("2-4-8-10".to_string(), 30.0);
        awards.insert("2-4-8-9".to_string(), 30.0);
        awards.insert("2-4-8-9-10".to_string(), 30.0);
        awards.insert("2-4-9".to_string(), 30.0);
        awards.insert("2-4-9-10".to_string(), 30.0);
        awards.insert("2-5-10".to_string(), 30.0);
        awards.insert("2-5-6".to_string(), 30.0);
        awards.insert("2-5-6-10".to_string(), 30.0);
        awards.insert("2-5-6-7".to_string(), 30.0);
        awards.insert("2-5-6-7-10".to_string(), 30.0);
        awards.insert("2-5-6-7-8".to_string(), 30.0);
        awards.insert("2-5-6-7-8-10".to_string(), 30.0);
        awards.insert("2-5-6-7-8-9".to_string(), 30.0);
        awards.insert("2-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-5-6-7-9".to_string(), 30.0);
        awards.insert("2-5-6-7-9-10".to_string(), 30.0);
        awards.insert("2-5-6-8".to_string(), 30.0);
        awards.insert("2-5-6-8-10".to_string(), 30.0);
        awards.insert("2-5-6-8-9".to_string(), 30.0);
        awards.insert("2-5-6-8-9-10".to_string(), 30.0);
        awards.insert("2-5-6-9".to_string(), 30.0);
        awards.insert("2-5-6-9-10".to_string(), 30.0);
        awards.insert("2-5-7".to_string(), 30.0);
        awards.insert("2-5-7-10".to_string(), 30.0);
        awards.insert("2-5-7-8".to_string(), 30.0);
        awards.insert("2-5-7-8-10".to_string(), 30.0);
        awards.insert("2-5-7-8-9".to_string(), 30.0);
        awards.insert("2-5-7-8-9-10".to_string(), 30.0);
        awards.insert("2-5-7-9".to_string(), 30.0);
        awards.insert("2-5-7-9-10".to_string(), 30.0);
        awards.insert("2-5-8-10".to_string(), 30.0);
        awards.insert("2-5-8-9-10".to_string(), 30.0);
        awards.insert("2-5-9-10".to_string(), 30.0);
        awards.insert("2-6".to_string(), 30.0);
        awards.insert("2-6-10".to_string(), 30.0);
        awards.insert("2-6-7".to_string(), 30.0);
        awards.insert("2-6-7-10".to_string(), 30.0);
        awards.insert("2-6-7-8".to_string(), 30.0);
        awards.insert("2-6-7-8-10".to_string(), 30.0);
        awards.insert("2-6-7-8-9".to_string(), 30.0);
        awards.insert("2-6-7-8-9-10".to_string(), 30.0);
        awards.insert("2-6-7-9".to_string(), 30.0);
        awards.insert("2-6-7-9-10".to_string(), 30.0);
        awards.insert("2-6-8".to_string(), 30.0);
        awards.insert("2-6-8-10".to_string(), 30.0);
        awards.insert("2-6-8-9".to_string(), 30.0);
        awards.insert("2-6-8-9-10".to_string(), 30.0);
        awards.insert("2-6-9".to_string(), 30.0);
        awards.insert("2-6-9-10".to_string(), 30.0);
        awards.insert("2-7-10".to_string(), 30.0);
        awards.insert("2-7-8".to_string(), 30.0);
        awards.insert("2-7-8-10".to_string(), 30.0);
        awards.insert("2-7-8-9".to_string(), 30.0);
        awards.insert("2-7-8-9-10".to_string(), 30.0);
        awards.insert("2-7-9".to_string(), 30.0);
        awards.insert("2-7-9-10".to_string(), 30.0);
        awards.insert("2-8-9".to_string(), 30.0);
        awards.insert("2-8-9-10".to_string(), 30.0);
        awards.insert("2-9-10".to_string(), 30.0);
        awards.insert("3-4".to_string(), 30.0);
        awards.insert("3-4-10".to_string(), 30.0);
        awards.insert("3-4-5".to_string(), 30.0);
        awards.insert("3-4-5-10".to_string(), 30.0);
        awards.insert("3-4-5-6".to_string(), 30.0);
        awards.insert("3-4-5-6-10".to_string(), 30.0);
        awards.insert("3-4-5-6-7".to_string(), 30.0);
        awards.insert("3-4-5-6-7-10".to_string(), 30.0);
        awards.insert("3-4-5-6-7-8".to_string(), 30.0);
        awards.insert("3-4-5-6-7-8-10".to_string(), 30.0);
        awards.insert("3-4-5-6-7-8-9".to_string(), 30.0);
        awards.insert("3-4-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("3-4-5-6-7-9".to_string(), 30.0);
        awards.insert("3-4-5-6-7-9-10".to_string(), 30.0);
        awards.insert("3-4-5-6-8".to_string(), 30.0);
        awards.insert("3-4-5-6-8-10".to_string(), 30.0);
        awards.insert("3-4-5-6-8-9".to_string(), 30.0);
        awards.insert("3-4-5-6-8-9-10".to_string(), 30.0);
        awards.insert("3-4-5-6-9".to_string(), 30.0);
        awards.insert("3-4-5-6-9-10".to_string(), 30.0);
        awards.insert("3-4-5-7".to_string(), 30.0);
        awards.insert("3-4-5-7-10".to_string(), 30.0);
        awards.insert("3-4-5-7-8".to_string(), 30.0);
        awards.insert("3-4-5-7-8-10".to_string(), 30.0);
        awards.insert("3-4-5-7-8-9".to_string(), 30.0);
        awards.insert("3-4-5-7-8-9-10".to_string(), 30.0);
        awards.insert("3-4-5-7-9".to_string(), 30.0);
        awards.insert("3-4-5-7-9-10".to_string(), 30.0);
        awards.insert("3-4-5-8".to_string(), 30.0);
        awards.insert("3-4-5-8-10".to_string(), 30.0);
        awards.insert("3-4-5-8-9".to_string(), 30.0);
        awards.insert("3-4-5-8-9-10".to_string(), 30.0);
        awards.insert("3-4-5-9".to_string(), 30.0);
        awards.insert("3-4-5-9-10".to_string(), 30.0);
        awards.insert("3-4-6".to_string(), 30.0);
        awards.insert("3-4-6-10".to_string(), 30.0);
        awards.insert("3-4-6-7".to_string(), 30.0);
        awards.insert("3-4-6-7-10".to_string(), 30.0);
        awards.insert("3-4-6-7-8".to_string(), 30.0);
        awards.insert("3-4-6-7-8-10".to_string(), 30.0);
        awards.insert("3-4-6-7-8-9".to_string(), 30.0);
        awards.insert("3-4-6-7-8-9-10".to_string(), 30.0);
        awards.insert("3-4-6-7-9".to_string(), 30.0);
        awards.insert("3-4-6-7-9-10".to_string(), 30.0);
        awards.insert("3-4-6-8".to_string(), 30.0);
        awards.insert("3-4-6-8-10".to_string(), 30.0);
        awards.insert("3-4-6-8-9".to_string(), 30.0);
        awards.insert("3-4-6-8-9-10".to_string(), 30.0);
        awards.insert("3-4-6-9".to_string(), 30.0);
        awards.insert("3-4-6-9-10".to_string(), 30.0);
        awards.insert("3-4-7".to_string(), 30.0);
        awards.insert("3-4-7-10".to_string(), 30.0);
        awards.insert("3-4-7-8".to_string(), 30.0);
        awards.insert("3-4-7-8-10".to_string(), 30.0);
        awards.insert("3-4-7-8-9".to_string(), 30.0);
        awards.insert("3-4-7-8-9-10".to_string(), 30.0);
        awards.insert("3-4-7-9".to_string(), 30.0);
        awards.insert("3-4-7-9-10".to_string(), 30.0);
        awards.insert("3-4-8".to_string(), 30.0);
        awards.insert("3-4-8-10".to_string(), 30.0);
        awards.insert("3-4-8-9".to_string(), 30.0);
        awards.insert("3-4-8-9-10".to_string(), 30.0);
        awards.insert("3-4-9".to_string(), 30.0);
        awards.insert("3-4-9-10".to_string(), 30.0);
        awards.insert("3-5-10".to_string(), 30.0);
        awards.insert("3-5-6-7".to_string(), 30.0);
        awards.insert("3-5-6-7-10".to_string(), 30.0);
        awards.insert("3-5-6-7-8".to_string(), 30.0);
        awards.insert("3-5-6-7-8-10".to_string(), 30.0);
        awards.insert("3-5-6-7-8-9".to_string(), 30.0);
        awards.insert("3-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("3-5-6-7-9".to_string(), 30.0);
        awards.insert("3-5-6-7-9-10".to_string(), 30.0);
        awards.insert("3-5-7".to_string(), 30.0);
        awards.insert("3-5-7-10".to_string(), 30.0);
        awards.insert("3-5-7-8".to_string(), 30.0);
        awards.insert("3-5-7-8-10".to_string(), 30.0);
        awards.insert("3-5-7-8-9".to_string(), 30.0);
        awards.insert("3-5-7-8-9-10".to_string(), 30.0);
        awards.insert("3-5-7-9".to_string(), 30.0);
        awards.insert("3-5-7-9-10".to_string(), 30.0);
        awards.insert("3-5-8-10".to_string(), 30.0);
        awards.insert("3-5-8-9-10".to_string(), 30.0);
        awards.insert("3-5-9-10".to_string(), 30.0);
        awards.insert("3-6-7".to_string(), 30.0);
        awards.insert("3-6-7-10".to_string(), 30.0);
        awards.insert("3-6-7-8".to_string(), 30.0);
        awards.insert("3-6-7-8-10".to_string(), 30.0);
        awards.insert("3-6-7-8-9".to_string(), 30.0);
        awards.insert("3-6-7-8-9-10".to_string(), 30.0);
        awards.insert("3-6-7-9".to_string(), 30.0);
        awards.insert("3-6-7-9-10".to_string(), 30.0);
        awards.insert("3-6-8".to_string(), 30.0);
        awards.insert("3-6-8-10".to_string(), 30.0);
        awards.insert("3-6-8-9".to_string(), 30.0);
        awards.insert("3-6-8-9-10".to_string(), 30.0);
        awards.insert("3-7".to_string(), 30.0);
        awards.insert("3-7-10".to_string(), 30.0);
        awards.insert("3-7-8".to_string(), 30.0);
        awards.insert("3-7-8-10".to_string(), 30.0);
        awards.insert("3-7-8-9".to_string(), 30.0);
        awards.insert("3-7-8-9-10".to_string(), 30.0);
        awards.insert("3-7-9-10".to_string(), 30.0);
        awards.insert("3-8-10".to_string(), 30.0);
        awards.insert("3-8-9".to_string(), 30.0);
        awards.insert("3-8-9-10".to_string(), 30.0);
        awards.insert("3-9-10".to_string(), 30.0);
        awards.insert("4-10".to_string(), 30.0);
        awards.insert("4-5-10".to_string(), 30.0);
        awards.insert("4-5-6".to_string(), 30.0);
        awards.insert("4-5-6-10".to_string(), 30.0);
        awards.insert("4-5-6-7".to_string(), 30.0);
        awards.insert("4-5-6-7-10".to_string(), 30.0);
        awards.insert("4-5-6-7-8".to_string(), 30.0);
        awards.insert("4-5-6-7-8-10".to_string(), 30.0);
        awards.insert("4-5-6-7-8-9".to_string(), 30.0);
        awards.insert("4-5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("4-5-6-7-9".to_string(), 30.0);
        awards.insert("4-5-6-7-9-10".to_string(), 30.0);
        awards.insert("4-5-6-8".to_string(), 30.0);
        awards.insert("4-5-6-8-10".to_string(), 30.0);
        awards.insert("4-5-6-8-9".to_string(), 30.0);
        awards.insert("4-5-6-8-9-10".to_string(), 30.0);
        awards.insert("4-5-6-9".to_string(), 30.0);
        awards.insert("4-5-6-9-10".to_string(), 30.0);
        awards.insert("4-5-7".to_string(), 30.0);
        awards.insert("4-5-7-10".to_string(), 30.0);
        awards.insert("4-5-7-8".to_string(), 30.0);
        awards.insert("4-5-7-8-10".to_string(), 30.0);
        awards.insert("4-5-7-8-9".to_string(), 30.0);
        awards.insert("4-5-7-8-9-10".to_string(), 30.0);
        awards.insert("4-5-7-9".to_string(), 30.0);
        awards.insert("4-5-7-9-10".to_string(), 30.0);
        awards.insert("4-5-8".to_string(), 30.0);
        awards.insert("4-5-8-10".to_string(), 30.0);
        awards.insert("4-5-8-9".to_string(), 30.0);
        awards.insert("4-5-8-9-10".to_string(), 30.0);
        awards.insert("4-5-9".to_string(), 30.0);
        awards.insert("4-5-9-10".to_string(), 30.0);
        awards.insert("4-6-7-8".to_string(), 30.0);
        awards.insert("4-6-7-8-9".to_string(), 30.0);
        awards.insert("4-6-7-8-9-10".to_string(), 30.0);
        awards.insert("4-6-7-9".to_string(), 30.0);
        awards.insert("4-6-8".to_string(), 30.0);
        awards.insert("4-6-8-10".to_string(), 30.0);
        awards.insert("4-6-8-9".to_string(), 30.0);
        awards.insert("4-6-8-9-10".to_string(), 30.0);
        awards.insert("4-6-9".to_string(), 30.0);
        awards.insert("4-6-9-10".to_string(), 30.0);
        awards.insert("4-7-10".to_string(), 30.0);
        awards.insert("4-7-8-10".to_string(), 30.0);
        awards.insert("4-7-8-9".to_string(), 30.0);
        awards.insert("4-7-8-9-10".to_string(), 30.0);
        awards.insert("4-7-9".to_string(), 30.0);
        awards.insert("4-7-9-10".to_string(), 30.0);
        awards.insert("4-8-10".to_string(), 30.0);
        awards.insert("4-8-9".to_string(), 30.0);
        awards.insert("4-8-9-10".to_string(), 30.0);
        awards.insert("4-9".to_string(), 30.0);
        awards.insert("4-9-10".to_string(), 30.0);
        awards.insert("5-10".to_string(), 30.0);
        awards.insert("5-6-10".to_string(), 30.0);
        awards.insert("5-6-7".to_string(), 30.0);
        awards.insert("5-6-7-10".to_string(), 30.0);
        awards.insert("5-6-7-8".to_string(), 30.0);
        awards.insert("5-6-7-8-10".to_string(), 30.0);
        awards.insert("5-6-7-8-9".to_string(), 30.0);
        awards.insert("5-6-7-8-9-10".to_string(), 30.0);
        awards.insert("5-6-7-9".to_string(), 30.0);
        awards.insert("5-6-7-9-10".to_string(), 30.0);
        awards.insert("5-6-8".to_string(), 30.0);
        awards.insert("5-6-8-10".to_string(), 30.0);
        awards.insert("5-6-8-9".to_string(), 30.0);
        awards.insert("5-6-8-9-10".to_string(), 30.0);
        awards.insert("5-6-9".to_string(), 30.0);
        awards.insert("5-6-9-10".to_string(), 30.0);
        awards.insert("5-7".to_string(), 30.0);
        awards.insert("5-7-10".to_string(), 30.0);
        awards.insert("5-7-8".to_string(), 30.0);
        awards.insert("5-7-8-10".to_string(), 30.0);
        awards.insert("5-7-8-9".to_string(), 30.0);
        awards.insert("5-7-8-9-10".to_string(), 30.0);
        awards.insert("5-7-9".to_string(), 30.0);
        awards.insert("5-7-9-10".to_string(), 30.0);
        awards.insert("5-8-10".to_string(), 30.0);
        awards.insert("5-8-9-10".to_string(), 30.0);
        awards.insert("5-9-10".to_string(), 30.0);
        awards.insert("6-7".to_string(), 30.0);
        awards.insert("6-7-10".to_string(), 30.0);
        awards.insert("6-7-8".to_string(), 30.0);
        awards.insert("6-7-8-10".to_string(), 30.0);
        awards.insert("6-7-8-9".to_string(), 30.0);
        awards.insert("6-7-8-9-10".to_string(), 30.0);
        awards.insert("6-7-9".to_string(), 30.0);
        awards.insert("6-7-9-10".to_string(), 30.0);
        awards.insert("6-8".to_string(), 30.0);
        awards.insert("6-8-10".to_string(), 30.0);
        awards.insert("6-8-9".to_string(), 30.0);
        awards.insert("6-8-9-10".to_string(), 30.0);
        awards.insert("7-8-10".to_string(), 30.0);
        awards.insert("7-8-9".to_string(), 30.0);
        awards.insert("7-8-9-10".to_string(), 30.0);
        awards.insert("7-9-10".to_string(), 30.0);
        awards.insert("8-9-10".to_string(), 30.0);

        // 50% splits
        awards.insert("2-8-10".to_string(), 50.0);
        awards.insert("3-7-9".to_string(), 50.0);
        awards.insert("4-6".to_string(), 50.0);
        awards.insert("4-6-10".to_string(), 50.0);
        awards.insert("4-6-7".to_string(), 50.0);
        awards.insert("4-6-7-10".to_string(), 50.0);
        awards.insert("4-6-7-8-10".to_string(), 50.0);
        awards.insert("4-6-7-9-10".to_string(), 50.0);
        awards.insert("7-10".to_string(), 50.0);
        awards.insert("7-9".to_string(), 50.0);
        awards.insert("8-10".to_string(), 50.0);

        awards
    }
}
