use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::models::account::ClientAccount;
use crate::models::transaction::TransactionRecord;
use crate::models::transaction::TransactionStatus;

#[derive(Debug, Clone)]
enum TransactionMssage {
    Record(TransactionRecord),
    Terminate,
}

#[derive(Debug)]
pub struct TransactionEngineService {
    // Key: client ID, Value: ClientAccount
    pub client_account: Arc<Mutex<HashMap<u16, crate::models::account::ClientAccount>>>,
    // Key: transaction ID, Value: TransactionStatus
    pub processed_transactions: Arc<Mutex<HashMap<u32, TransactionStatus>>>,
}

impl Default for TransactionEngineService {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionEngineService {
    pub fn new() -> Self {
        TransactionEngineService {
            client_account: Arc::new(Mutex::new(HashMap::new())),
            processed_transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn begin_transactions_from_file(
        &mut self,
        file_path: &str,
        batch_mode: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = tokio::fs::File::open(file_path).await?;
        let file = tokio_util::compat::TokioAsyncReadCompatExt::compat(file);
        let mut rdr = csv_async::AsyncReaderBuilder::new()
            .trim(csv_async::Trim::All)
            .create_deserializer(file);

        let mut workers: HashMap<u16, mpsc::Sender<TransactionMssage>> = HashMap::new();
        let mut records = rdr.deserialize::<TransactionRecord>();

        let mut handles = Vec::new();
        while let Some(result) = records.next().await {
            let record = result?;
            let client_id = record.client;

            if !workers.contains_key(&client_id) {
                let (handle, tx) = self.spawn_worker(client_id, batch_mode).await?;
                handles.push(handle);
                workers.insert(client_id, tx);
            }

            if let Some(sender) = workers.get(&client_id) {
                sender.send(TransactionMssage::Record(record)).await.ok();
            }
        }

        for (_client_id, sender) in workers {
            sender.send(TransactionMssage::Terminate).await.ok();
        }

        for handle in handles {
            handle.await.ok();
        }

        Ok(())
    }

    async fn spawn_worker(
        &self,
        client_id: u16,
        batch_mode: bool,
    ) -> Result<
        (tokio::task::JoinHandle<()>, mpsc::Sender<TransactionMssage>),
        Box<dyn std::error::Error>,
    > {
        let (tx, mut rx) = mpsc::channel::<TransactionMssage>(100);
        let accounts = Arc::clone(&self.client_account);
        let processed = Arc::clone(&self.processed_transactions);

        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    TransactionMssage::Record(record) => {
                        let mut accounts_guard = accounts.lock().unwrap();
                        let mut processed_guard = processed.lock().unwrap();

                        let account = accounts_guard
                            .entry(client_id)
                            .or_insert_with(|| ClientAccount::new(client_id));
                        if batch_mode {
                            let result =
                                account.apply_transaction_record(&record, &mut processed_guard);
                            if let Err(err) = result {
                                panic!(
                                    "Error processing transaction for client {}: {}",
                                    client_id, err
                                );
                            }
                        } else {
                            let _ = account.apply_transaction_record(&record, &mut processed_guard);
                        }
                    }
                    TransactionMssage::Terminate => break,
                }
            }
        });

        Ok((handle, tx))
    }

    pub async fn print_client_accounts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());

        // Do we need sort?
        // No, but for testing purposes.
        let sorted_client_accounts = {
            let accounts_guard = self.client_account.lock().unwrap();
            let mut accounts: Vec<ClientAccount> = accounts_guard.values().cloned().collect();
            accounts.sort_by_key(|account| account.client);
            accounts
        };
        for account in &sorted_client_accounts {
            writer.serialize(account)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub async fn print_client_accounts_four_decimal_places(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = tokio::io::stdout();

        writer
            .write_all(b"client,available,held,total,locked\n")
            .await?;

        let sorted_client_accounts = {
            let accounts_guard = self.client_account.lock().unwrap();
            let mut accounts: Vec<ClientAccount> = accounts_guard.values().cloned().collect();
            accounts.sort_by_key(|account| account.client);
            accounts
        };
        for account in &sorted_client_accounts {
            let line = format!(
                "{},{:.4},{:.4},{:.4},{}\n",
                account.client,
                (account.available * 10000.0).round() / 10000.0,
                (account.held * 10000.0).round() / 10000.0,
                (account.total * 10000.0).round() / 10000.0,
                account.locked
            );
            writer.write_all(line.as_bytes()).await?;
        }
        writer.flush().await?;
        Ok(())
    }
}
