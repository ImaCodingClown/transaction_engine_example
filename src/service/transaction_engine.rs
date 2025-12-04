use std::collections::HashMap;

use crate::models::account::ClientAccount;
use crate::models::transaction::TransactionRecord;
use crate::models::transaction::TransactionStatus;

#[derive(Debug)]
pub struct TransactionEngineService {
    // Key: client ID, Value: ClientAccount
    pub client_account: std::collections::HashMap<u16, crate::models::account::ClientAccount>,
    // Key: transaction ID, Value: TransactionStatus
    pub processed_transactions: HashMap<u32, TransactionStatus>,
}

impl Default for TransactionEngineService {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionEngineService {
    pub fn new() -> Self {
        TransactionEngineService {
            client_account: std::collections::HashMap::new(),
            processed_transactions: HashMap::new(),
        }
    }

    pub fn process_transaction_record(
        &mut self,
        record: &TransactionRecord,
        batch_mode: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client_id = record.client;

        let client_account = self
            .client_account
            .entry(client_id)
            .or_insert(ClientAccount::new(client_id));
        if batch_mode {
            client_account.apply_transaction_record(record, &mut self.processed_transactions)?;
        } else {
            // In non-batch mode, we ignore errors for individual transactions
            let _ =
                client_account.apply_transaction_record(record, &mut self.processed_transactions);
        }
        Ok(())
    }

    pub fn begin_transactions_from_file(
        &mut self,
        file_path: &str,
        batch_mode: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(file_path)?;
        let mut transactions = Vec::new();

        for result in rdr.deserialize() {
            let record: crate::models::transaction::TransactionRecord = result?;
            self.process_transaction_record(&record, batch_mode)?;
            transactions.push(record);
        }

        Ok(())
    }

    pub fn print_client_accounts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());

        // Do we need sort?
        let sorted_client_accounts = {
            let mut accounts: Vec<&ClientAccount> = self.client_account.values().collect();
            accounts.sort_by_key(|account| account.client);
            accounts
        };
        for account in sorted_client_accounts {
            wtr.serialize(account)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
