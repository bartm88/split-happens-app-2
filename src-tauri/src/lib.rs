use serde_json::json;
use std::collections::HashSet;
use std::time::Instant;
use storage::sheets_dao::{Balance, SheetsDao, Transaction};
use tauri_plugin_store::StoreExt;

mod storage;

const DEMO_SHEET_ID: &str = "1SIvYTqRcno-BxMWZAWNcw208N3WREZRRcPzjn_ftUYo";

// Can we have a singleton instance of the dao?
// Probably, but based on the log timing, these rarely takes more than 2ms.

fn get_sheet_id_from_store(app: tauri::AppHandle) -> String {
    let store = app.store("store.json").expect("Failed to open store");
    let sheet_id = store.get("sheet-id").unwrap_or_default();
    let sheet_id = sheet_id.get("value").expect("Failed to get sheet id");
    let sheet_id = sheet_id
        .as_str()
        .expect("Failed to get sheet id as string")
        .to_string();
    sheet_id
}

#[tauri::command]
async fn balances(app: tauri::AppHandle) -> Result<Vec<Balance>, ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    let result = sheets_dao.get_balances().await;
    log::info!("get_balances operation took {:?}", start.elapsed());
    Ok(result)
}

#[tauri::command]
async fn names(app: tauri::AppHandle) -> Result<Vec<String>, ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    let result = sheets_dao.get_names().await;
    log::info!("get_names operation took {:?}", start.elapsed());
    Ok(result)
}

#[tauri::command]
async fn transactions(app: tauri::AppHandle, count: usize) -> Result<Vec<Transaction>, ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    let in_order = sheets_dao.get_last_n_transactions(count).await;
    log::info!(
        "get_last_n_transactions operation took {:?}",
        start.elapsed()
    );
    Ok(in_order.into_iter().rev().collect())
}

#[tauri::command]
async fn remove_last_transaction(app: tauri::AppHandle) -> Result<(), ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    sheets_dao.remove_last_transaction().await;
    log::info!(
        "remove_last_transaction operation took {:?}",
        start.elapsed()
    );
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn create_split(app: tauri::AppHandle, name: &str, split_string: &str) -> Result<(), ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    sheets_dao
        .add_split(name.to_string(), split_string.to_string())
        .await;
    log::info!("add_split operation took {:?}", start.elapsed());
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
async fn convert_split(app: tauri::AppHandle, name: &str, split_string: &str) -> Result<(), ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    sheets_dao
        .add_conversion(name.to_string(), split_string.to_string())
        .await;
    log::info!("add_conversion operation took {:?}", start.elapsed());
    Ok(())
}

#[tauri::command]
async fn get_valid_splits(app: tauri::AppHandle) -> Result<HashSet<String>, ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    let sheets_dao = SheetsDao::new(sheet_id).await;
    log::info!("SheetsDao initialization took {:?}", start.elapsed());

    let start = Instant::now();
    let result = sheets_dao
        .get_split_awards()
        .await
        .iter()
        .map(|(k, _v)| k.clone())
        .collect();
    log::info!("get_split_awards operation took {:?}", start.elapsed());
    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
async fn set_sheet_id(app: tauri::AppHandle, sheet_id: &str) -> Result<(), ()> {
    let start: Instant = Instant::now();
    let store = app.store("store.json").expect("Failed to open store");
    store.set("sheet-id", json!({ "value": sheet_id }));
    store.save().expect("Failed to save store");
    log::info!("set_sheet_id operation took {:?}", start.elapsed());
    store.close_resource();
    Ok(())
}

#[tauri::command]
async fn get_sheet_id(app: tauri::AppHandle) -> Result<String, ()> {
    let start = Instant::now();
    let sheet_id = get_sheet_id_from_store(app);
    log::info!("get_sheet_id operation took {:?}", start.elapsed());
    Ok(sheet_id)
}

#[tauri::command]
async fn set_demo_sheet_id(app: tauri::AppHandle) -> Result<(), ()> {
    let start = Instant::now();
    let store = app.store("store.json").expect("Failed to open store");
    store.set("sheet-id", json!({ "value": DEMO_SHEET_ID }));
    store.save().expect("Failed to save store");
    log::info!("set_demo_sheet_id operation took {:?}", start.elapsed());
    store.close_resource();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            balances,
            names,
            transactions,
            remove_last_transaction,
            create_split,
            convert_split,
            get_valid_splits,
            set_sheet_id,
            get_sheet_id,
            set_demo_sheet_id
        ])
        .setup(|app| {
            let store = app.store("store.json")?;
            store.clear();
            log::info!("Current store: {:?}", store.entries());
            if store.get("sheet-id").is_none() {
                log::info!("Setting sheet id to {}", DEMO_SHEET_ID);
                store.set("sheet-id", json!({ "value": DEMO_SHEET_ID }));
            } else {
                log::info!(
                    "Sheet id already set to {}",
                    store
                        .get("sheet-id")
                        .unwrap()
                        .get("value")
                        .unwrap()
                        .as_str()
                        .unwrap()
                );
            }
            store.save()?;
            store.close_resource();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
