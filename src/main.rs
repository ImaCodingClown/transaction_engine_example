pub mod error;
pub mod models;
pub mod service;
pub mod utils;

#[cfg(test)]
mod test;

use crate::service::transaction_engine;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Path to the CSV file containing transactions
    #[clap(value_parser, value_parser=utils::validate_file_path)]
    file_path: String,
    /// Enable batch mode processing
    /// In batch mode, any error in processing transactions will halt the entire processing.
    #[clap(long, action)]
    batch: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut transaction_engine = transaction_engine::TransactionEngineService::new();
    transaction_engine
        .begin_transactions_from_file(&args.file_path, args.batch)
        .await?;
    // transaction_engine.print_client_accounts().await?;
    transaction_engine
        .print_client_accounts_four_decimal_places()
        .await?; // for four decimal places.
    Ok(())
}
