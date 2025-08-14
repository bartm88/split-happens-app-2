# Storage Abstraction for Split Happens

## Overview
This project now uses a trait-based storage abstraction that allows for drop-in replacement of storage implementations.

## Available Implementations

### 1. Google Sheets DAO (`sheets_dao`)
- The original implementation that connects to Google Sheets
- Requires a valid Google Sheets ID configured in the app
- Used in production

### 2. In-Memory DAO (`memory_dao`)
- A test implementation that stores data in memory
- Pre-seeded with test data including:
  - 4 test users: Alice, Bob, Charlie, Dana (plus Pot)
  - Valid splits with award percentages
  - Sample transactions including splits and conversions
- Perfect for testing and development

## Usage

To switch between implementations, modify the `USE_MEMORY_STORAGE` constant in `src/lib.rs`:

```rust
const USE_MEMORY_STORAGE: bool = true;  // Use in-memory storage
const USE_MEMORY_STORAGE: bool = false; // Use Google Sheets storage
```

## StorageDao Trait

The `StorageDao` trait defines the following methods:

```rust
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
```

## Adding New Implementations

To add a new storage implementation (e.g., DynamoDB):

1. Create a new module in `src/storage/` (e.g., `dynamodb_dao.rs`)
2. Implement the `StorageDao` trait for your struct
3. Update the factory function in `lib.rs` to support your implementation
4. Add any necessary dependencies to `Cargo.toml`

Example:
```rust
use super::{Balance, StorageDao, Transaction};
use async_trait::async_trait;

pub struct DynamoDbDao {
    // Your implementation details
}

#[async_trait]
impl StorageDao for DynamoDbDao {
    // Implement all trait methods
}
```

## Test Data (In-Memory Implementation)

The in-memory implementation includes:
- Names: Alice, Bob, Charlie, Dana, Pot
- Valid splits: 1-2-3, 2-3, 3-6, 4-5, 4-5-6, 5-6, 4-7-10, 6-7-10, 7-10
- Sample transactions with realistic data