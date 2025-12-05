use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::transaction::{TransactionRecord, TransactionStatus, TransactionType},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientAccount {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl ClientAccount {
    pub fn new(client: u16) -> Self {
        ClientAccount {
            client,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn apply_transaction_record(
        &mut self,
        transaction: &TransactionRecord,
        processed_transactions: &mut HashMap<u32, TransactionStatus>,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.locked {
            return Err(AppError::AccountLocked)?;
        }
        match transaction.r#type {
            TransactionType::Deposit => {
                if processed_transactions.get(&transaction.tx).is_some() {
                    return Err(AppError::DuplicateRecord)?;
                }
                if let Some(amount) = transaction.amount {
                    self.deposit(amount)?;
                    processed_transactions.insert(
                        transaction.tx,
                        TransactionStatus {
                            deposited: transaction.clone(),
                            disputed: false,
                            charged_back: false,
                            resolved: false,
                        },
                    );
                } else {
                    Err(AppError::InvalidTransactionFundAmount)?;
                }
                Ok(self)
            }
            TransactionType::Withdrawal => {
                if let Some(amount) = transaction.amount {
                    self.withdraw(amount)?;
                } else {
                    Err(AppError::InvalidTransactionFundAmount)?;
                }
                Ok(self)
            }
            TransactionType::Dispute => {
                if let Some(status) = processed_transactions.get_mut(&transaction.tx) {
                    if !status.disputed {
                        let amount = status
                            .deposited
                            .amount
                            .ok_or(AppError::InvalidTransactionFundAmount)?;
                        self.dispute(amount)?;
                        status.disputed = true;
                    } else {
                        Err(AppError::DisputeAlreadyExists)?;
                    }
                }
                Ok(self)
            }
            TransactionType::Resolve => {
                if let Some(status) = processed_transactions.get_mut(&transaction.tx) {
                    if status.disputed && !status.resolved {
                        let amount = status
                            .deposited
                            .amount
                            .ok_or(AppError::InvalidTransactionFundAmount)?;
                        self.resolve(amount)?;
                        status.resolved = true;
                        status.disputed = false;
                    }
                }
                Ok(self)
            }
            TransactionType::Chargeback => {
                if let Some(status) = processed_transactions.get_mut(&transaction.tx) {
                    if status.disputed && !status.charged_back {
                        let amount = status
                            .deposited
                            .amount
                            .ok_or(AppError::InvalidTransactionFundAmount)?;
                        self.chargeback(amount)?;
                        status.charged_back = true;
                        status.disputed = false;
                    }
                }
                Ok(self)
            }
        }
    }
    pub fn deposit(&mut self, amount: f64) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.available += amount;
        self.total += amount;
        Ok(self)
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            Ok(self)
        } else {
            Err(AppError::NotEnoughFunds)?
        }
    }

    pub fn dispute(&mut self, amount: f64) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            Ok(self)
        } else {
            Err(AppError::NotEnoughFunds)?
        }
    }

    pub fn resolve(&mut self, amount: f64) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.held >= amount {
            self.held -= amount;
            self.available += amount;
            Ok(self)
        } else {
            Err(AppError::NotEnoughFunds)?
        }
    }

    pub fn chargeback(&mut self, amount: f64) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.held >= amount {
            self.held -= amount;
            self.total -= amount;
            self.locked = true;
            Ok(self)
        } else {
            Err(AppError::NotEnoughFunds)?
        }
    }
}
