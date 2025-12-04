use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionRecord {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transactions: Vec<TransactionRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub deposited: TransactionRecord,
    pub disputed: bool,
    pub charged_back: bool,
    pub resolved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
