use super::{Balance, StorageDao, Transaction};
use async_trait::async_trait;
use std::{collections::HashMap, time::SystemTime};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use google_sheets4::{
    api::{CellData, ClearValuesRequest, RowData, Spreadsheet, ValueRange},
    hyper::Response,
    hyper_rustls::{self, HttpsConnector},
    hyper_util::{self, client::legacy::connect::HttpConnector},
    yup_oauth2::{self, ServiceAccountAuthenticator},
    Sheets,
};
use http_body_util::combinators::BoxBody;
use serde_json::json;

const POT: &str = "Pot";

pub struct SheetsDao {
    sheet_id: String,
    sheets: Sheets<HttpsConnector<HttpConnector>>,
}

impl SheetsDao {
    pub async fn new(sheet_id: String) -> Self {
        let creds = yup_oauth2::parse_service_account_key(
            crate::secrets::SERVICE_ACCOUNT_KEY_STRING_TEMPLATE,
        )
        .expect("Can't read credential, an error occurred");

        // This seems silly; the sheets constructor and the account authenticator disagree on the second generic parameter type so lets make two of them...
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_webpki_roots()
                        .https_or_http()
                        .enable_http1()
                        .build(),
                );

        let sa_client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_webpki_roots()
                        .https_or_http()
                        .enable_http1()
                        .build(),
                );

        let sa = ServiceAccountAuthenticator::builder(creds.clone())
            .hyper_client(sa_client)
            .build()
            .await
            .expect("There was an error, trying to build connection with authenticator");

        let sheets = Sheets::new(client, sa);
        Self { sheet_id, sheets }
    }

    pub async fn get_names(&self) -> Vec<String> {
        let names_response = self
            .sheets
            .spreadsheets()
            .get(self.sheet_id.as_str())
            .add_ranges("Ranges!A:A")
            .include_grid_data(true)
            .doit()
            .await;
        let row_data = self.get_row_data(names_response);
        let names: Vec<String> = row_data
            .iter()
            .map(|datum: &google_sheets4::api::RowData| {
                let row_cells = datum.values.as_ref().expect("No cell data");
                self.get_string_cell_value(row_cells, 0)
            })
            .collect();
        names
    }

    pub async fn get_balances(&self) -> Vec<Balance> {
        let balances_response = self
            .sheets
            .spreadsheets()
            .get(self.sheet_id.as_str())
            .add_ranges("Summary!A:B")
            .include_grid_data(true)
            .doit()
            .await;
        let row_data = self.get_row_data(balances_response);
        let name_balances: Vec<Balance> = row_data
            .iter()
            .map(|datum: &RowData| {
                let row_cells = datum.values.as_ref().expect("No cell data");
                Balance {
                    name: self.get_string_cell_value(row_cells, 0),
                    amount: self.get_string_cell_value(row_cells, 1),
                }
            })
            .collect();
        name_balances
    }

    pub async fn add_split(&self, name: String, split: String) {
        let pot_balance = self
            .get_balances()
            .await
            .iter()
            .filter(|name_bal| name_bal.name == POT)
            .map(|name_bal| name_bal.amount.clone())
            .next()
            .expect("Couldn't find pot balance.");
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();
        let req = ValueRange {
            major_dimension: None,
            range: None,
            values: Some(vec![vec![
                json!(name),
                json!(POT.to_string()),
                json!(1),
                json!(split),
                json!(now_string),
                json!(pot_balance),
                json!(today_string),
            ]]),
        };
        let _response = self
            .sheets
            .spreadsheets()
            .values_append(req.clone(), self.sheet_id.as_str(), "Transactions")
            .value_input_option("USER_ENTERED")
            .doit()
            .await;

        let _response = self
            .sheets
            .spreadsheets()
            .values_append(req, self.sheet_id.as_str(), "Activity Log")
            .value_input_option("USER_ENTERED")
            .doit()
            .await;
    }

    pub async fn add_conversion(&self, name: String, split: String) {
        let pot_balance: f64 = self
            .get_balances()
            .await
            .iter()
            .filter(|name_bal| name_bal.name == POT)
            .map(|name_bal| name_bal.amount.clone())
            .next()
            .expect("Couldn't find pot balance.")
            .parse()
            .expect("Couldn't parse value");
        let split_awards = self.get_split_awards().await;
        let award_multiplier_percent = split_awards
            .get(&split)
            .expect(format!("Invalid split: {}", split).as_str());
        let award = (pot_balance * award_multiplier_percent).round() / 100.0;
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();
        let req = ValueRange {
            major_dimension: None,
            range: None,
            values: Some(vec![vec![
                json!(POT.to_string()),
                json!(name),
                json!(award),
                json!(split),
                json!(now_string),
                json!(pot_balance),
                json!(today_string),
            ]]),
        };
        let _response = self
            .sheets
            .spreadsheets()
            .values_append(req.clone(), self.sheet_id.as_str(), "Transactions")
            .value_input_option("USER_ENTERED")
            .doit()
            .await;

        let _response = self
            .sheets
            .spreadsheets()
            .values_append(req, self.sheet_id.as_str(), "Activity Log")
            .value_input_option("USER_ENTERED")
            .doit()
            .await;
    }

    pub async fn get_split_awards(&self) -> HashMap<String, f64> {
        let split_awards_response = self
            .sheets
            .spreadsheets()
            .get(self.sheet_id.as_str())
            .add_ranges("Split Awards!A:B")
            .include_grid_data(true)
            .doit()
            .await;
        let row_data = self.get_row_data(split_awards_response);
        let split_awards: HashMap<String, f64> =
            HashMap::from_iter(row_data.iter().map(|datum: &google_sheets4::api::RowData| {
                let row_cells = datum.values.as_ref().expect("No cell data");
                (
                    self.get_string_cell_value(row_cells, 0),
                    self.get_f64_cell_value(row_cells, 1),
                )
            }));
        split_awards
    }

    pub async fn remove_last_transaction(&self) {
        let last_row_number = self.get_transaction_count_fast().await;

        let _result = self
            .sheets
            .spreadsheets()
            .values_clear(
                ClearValuesRequest::default(),
                self.sheet_id.as_str(),
                format!("Transactions!{}:{}", last_row_number, last_row_number).as_str(),
            )
            .doit()
            .await;

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now_string = now.format("%-m/%-d/%Y, %l:%M:%S %p UTC").to_string();
        let today_string = now.format("%-m/%-d/%Y").to_string();
        let req = ValueRange {
            major_dimension: None,
            range: None,
            values: Some(vec![vec![
                json!("Undo"),
                json!("Undo"),
                json!("Undo"),
                json!("Undo"),
                json!(now_string),
                json!("Undo"),
                json!(today_string),
            ]]),
        };
        let _response = self
            .sheets
            .spreadsheets()
            .values_append(req, self.sheet_id.as_str(), "Activity Log")
            .value_input_option("USER_ENTERED")
            .doit()
            .await;
    }

    pub async fn get_last_n_transactions(&self, n: usize) -> Vec<Transaction> {
        let last_row_number = self.get_transaction_count_fast().await;

        let transactions_response = self
            .sheets
            .spreadsheets()
            .get(self.sheet_id.as_str())
            .add_ranges(
                format!(
                    "Transactions!{}:{}",
                    last_row_number - n + 1,
                    last_row_number
                )
                .as_str(),
            )
            .include_grid_data(true)
            .doit()
            .await;
        let row_data = self.get_row_data(transactions_response);
        let transactions: Vec<Transaction> = row_data
            .iter()
            .map(|datum: &RowData| {
                let row_cells = datum.values.as_ref().expect("No cell data");
                Transaction {
                    creditor: self.get_string_cell_value(row_cells, 0),
                    debtor: self.get_string_cell_value(row_cells, 1),
                    amount: self.get_f64_cell_value(row_cells, 2),
                    split: self.get_string_cell_value(row_cells, 3),
                    time: self.get_string_cell_value(row_cells, 4),
                    pot_amount: self.get_f64_cell_value(row_cells, 5),
                    date: self.get_string_cell_value(row_cells, 6),
                }
            })
            .collect();
        transactions
    }

    async fn get_transaction_count_fast(&self) -> usize {
        let metadata_response = self
            .sheets
            .spreadsheets()
            .get(self.sheet_id.as_str())
            .add_ranges("Metadata!B1")
            .include_grid_data(true)
            .doit()
            .await;
        let row_data = self.get_row_data(metadata_response);
        let metadata_cells = row_data
            .get(0)
            .expect("y no metadata")
            .values
            .as_ref()
            .expect("y no values");
        self.get_string_cell_value(metadata_cells, 0)
            .parse()
            .expect("parse fail")
    }

    fn get_row_data(
        &self,
        response: Result<
            (
                Response<BoxBody<Bytes, google_sheets4::hyper::Error>>,
                Spreadsheet,
            ),
            google_sheets4::Error,
        >,
    ) -> Vec<RowData> {
        response
            .as_ref()
            .expect("No result")
            .1
            .sheets
            .as_ref()
            .expect("No sheets")
            .get(0)
            .expect("No sheet")
            .data
            .as_ref()
            .expect("No grid data")
            .get(0)
            .expect("No grid datum")
            .row_data
            .as_ref()
            .expect("No row data")
            .clone()
    }

    fn get_string_cell_value(&self, row_cells: &Vec<CellData>, cell_index: usize) -> String {
        row_cells
            .get(cell_index)
            .expect("No cell datum")
            .formatted_value
            .clone()
            .expect("No formatted value")
    }

    fn get_f64_cell_value(&self, row_cells: &Vec<CellData>, cell_index: usize) -> f64 {
        row_cells
            .get(cell_index)
            .expect("No cell datum")
            .effective_value
            .clone()
            .expect("No effective value")
            .number_value
            .expect("No number value")
    }
}

#[async_trait]
impl StorageDao for SheetsDao {
    async fn get_names(&self) -> Vec<String> {
        self.get_names().await
    }

    async fn get_balances(&self) -> Vec<Balance> {
        self.get_balances().await
    }

    async fn get_last_n_transactions(&self, n: usize) -> Vec<Transaction> {
        self.get_last_n_transactions(n).await
    }

    async fn remove_last_transaction(&self) {
        self.remove_last_transaction().await
    }

    async fn add_split(&self, name: String, split: String) {
        self.add_split(name, split).await
    }

    async fn add_conversion(&self, name: String, split: String) {
        self.add_conversion(name, split).await
    }

    async fn get_split_awards(&self) -> HashMap<String, f64> {
        self.get_split_awards().await
    }
}
