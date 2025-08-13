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
use serde::{Deserialize, Serialize};
use serde_json::json;

const POT: &str = "Pot";

#[derive(Debug, Deserialize, Serialize)]
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
pub struct SheetsDao {
    sheet_id: String,
    sheets: Sheets<HttpsConnector<HttpConnector>>,
}

const SERVICE_ACCOUNT_KEY_STRING: &str = r#"
{
  "type": "service_account",
  "project_id": "splithappens",
  "private_key_id": "d3b1fd20bbad00d0ba9a5681d2ca46dd83c683f2",
  "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQCBmGkp3HMeKZc1\n2RDdtIs+ocYZ4zs4H4KhSM53DTIsDzqzgkJ9OoKW2kNXPycasBHylv2c4M6XNh4O\nSi8qzX2Aa4d6M2/ioVUhzFRVfZ73mm05b6NJIADLIP1MDyKPJsbiriK5NaJ1QB/k\nmt4XvaPOA9AAnHLS36CEi/pnougthoL3wZXe1f/n/O22kEnNnhYPYY7Bo5F+cJjT\nIhHlLz3jaonmvcZlTX7SDsYXIgaq7SWrS+MVdenuOt4KC2kXgtD1KUyXoV/fL94t\nEGObIShSHEazsgvsjuMbludPccZUQJgSNv/+PGOSwLrFhx+bEXrEVIuH2ymDuxxJ\n7fkD3xn9AgMBAAECggEAHyxey5uEM3K0tVbAGFyIDCpU4UBIK/FkdlBxnbCWa4PE\nTo7cWIw9AnWBMlj2GpTU1GJeoiBvgW8anrxYiTbB5CS5g3shBEZjmIwykxfzJ8NF\nV8YTZJEmr7YBSAzx9rZQrBFQN/X/XK3ungpktxg+cV14aNU5R6FUVt3m/4vvxpdX\ng7pmCiWAuI4y8uYbgLpgBms/U8LVgMOUkS/ecyc8ihodG7nN2kZRHPUj3haFi0+a\n4BVR3cqIm15KWZMAvCoCLcAdHG51cz8ttluM1BxDK/ezZOrnYRVaWIl0u9KMfMA6\n8FsCb5aoE0eKwqpNjOCxGlTi9Lezub0cGe562+pmXwKBgQC2kw0mcJbJja3K0ATX\nLGujvglvSQCtoKeSw+TLh7966+HUXf0n4IsrBgIOrVu4x+K5c7CD7ALuYLfyHqbw\nSoZxDSgvU/K+HBobfzWF0+6Y+AHBSz7xBQyoUjWvLDdcGSWM+Z+438hGMpExW1SX\nCF6FmOkBTu/xh8hVJtKAxml+6wKBgQC1tuZoEMZCFaK37ryD+9ZNYJirRdaItP/l\n5uUBZcOf92KoXfCONp07KohECABVDQ6HNiZ+AZFX1zjT1tgpZ8vrgkdE5Rz0HWmW\nQbcOZztGlt0sH7wN1xqBCdXsuipPFzKYTyWRRdkR77ZcTr6K9Q4K6+CBQbfCFwFW\nuMwYxcggtwKBgQCq/EMnifAd6RsnQvQVlJtmaXaqi2MgOMJXmDCxUHvKohkIa9HI\nQ+nyLBlHJ6IsBr9WUXuxwRnpqBj9nylXB2SrgdftoyBGXUkyEUvN/vKIvlPedBsJ\nXGJDTWLLoIxkK4TYZ3vnh2UoIPmLkO5C2Gq1kcQ+HnBm8nRzEv237pokawKBgQCh\nQp9W2wwWuXyeHo/N0UBtirvxwxiQWZB/RlkU1Gq3G6PCJxvEGVOPnj8voKoq0FuE\nQtoGGP4TJjyYQqGynRqq9gKpcWoweamqXsdFUPeZvWiqL7+DyNEMkt32J4BEkCGm\naRa9xW7OLB157afLSY4cwxeJnfilliTqATWfBmaEIQKBgQC1JVXT9WPTTtgQEgZb\nJGeXiyHIxDIMfBuC2sebomGDk0h6wZwDrxeAFw+/3gQGtY/iRbc6mAE/D7Q7gX2u\nVTmVbytqiVK/PVTNfDnzWAQgFcHKBBd940Iu+SLp78H0i2V4l33naKLjcCSrlkHx\n1bhJR4ESsW40T/9RRst2mFY70w==\n-----END PRIVATE KEY-----\n",
  "client_email": "splithappens@splithappens.iam.gserviceaccount.com",
  "client_id": "108179188177514035576",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/splithappens%40splithappens.iam.gserviceaccount.com",
  "universe_domain": "googleapis.com"
}"#;

impl SheetsDao {
    pub async fn new(sheet_id: String) -> Self {
        let creds = yup_oauth2::parse_service_account_key(SERVICE_ACCOUNT_KEY_STRING)
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
