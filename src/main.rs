pub mod error;
pub mod models;
pub mod service;
pub mod utils;

#[cfg(test)]
mod test;

use crate::{service::transaction_engine, utils::validate_args};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    validate_args(args.clone())?;

    let file_name = &args[1];

    let batch_mode = args.len() > 2;

    let mut transaction_engine = transaction_engine::TransactionEngineService::new();
    transaction_engine.begin_transactions_from_file(file_name, batch_mode)?;
    transaction_engine.print_client_accounts()?;

    Ok(())
}
