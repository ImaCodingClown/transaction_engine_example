use std::{error::Error, fmt};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    MissingFileArgument,
    InvalidFileFormat,
    TooManyArguments,
    WrongArgument(String),
    InvalidTransactionFundAmount,
    NotEnoughFunds,
    AccountLocked,
    DuplicateRecord,
    DisputeAlreadyExists,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingFileArgument => write!(f, "File argument is missing"),
            AppError::InvalidFileFormat => write!(f, "Invalid file format"),
            AppError::TooManyArguments => write!(f, "Too many arguments provided"),
            AppError::WrongArgument(arg) => write!(f, "Wrong argument provided: {arg}"),
            AppError::InvalidTransactionFundAmount => write!(f, "Invalid amount for transaction"),
            AppError::NotEnoughFunds => write!(f, "Not enough funds for transaction"),
            AppError::AccountLocked => write!(f, "Account is locked"),
            AppError::DuplicateRecord => write!(f, "Duplicate transaction record"),
            AppError::DisputeAlreadyExists => {
                write!(f, "Dispute already exists for this transaction")
            }
        }
    }
}

impl Error for AppError {}
