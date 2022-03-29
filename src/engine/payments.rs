use crate::engine::errors::PaymentError;
use crate::engine::ingestion::PaymentsQueue;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct PaymentsProcessor {
    payments_queue: PaymentsQueue,
    account_service: AccountService,
}

impl PaymentsProcessor {
    pub fn new(payments_queue: PaymentsQueue, account_service: AccountService) -> Self {
        Self {
            payments_queue,
            account_service,
        }
    }

    pub async fn start(&self) -> Result<(), PaymentError> {
        while let Some(transaction_string) = self.payments_queue.get_transaction() {
            let transaction = Transaction::from_str(&transaction_string)?;
            self.account_service
                .process_transaction(transaction)
                .await?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl FromStr for TransactionType {
    type Err = PaymentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => Ok(TransactionType::Deposit),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            "dispute" => Ok(TransactionType::Dispute),
            "resolve" => Ok(TransactionType::Resolve),
            "chargeback" => Ok(TransactionType::Chargeback),
            _ => Err(Self::Err::PaymentProcessingError(format!(
                "Invalid transaction type: {}",
                s.to_string()
            ))),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transaction {
    transaction_type: TransactionType,
    client_id: u16,
    transaction_id: u32,
    amount: f32,
    under_dispute: bool,
}

impl FromStr for Transaction {
    type Err = PaymentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(",").collect();
        if parts.len() < 4 {
            return Err(PaymentError::PaymentProcessingError(
                "Invalid input data".to_string(),
            ));
        }

        let transaction_type = TransactionType::from_str(parts[0])?;
        let client_id = parts[1].parse::<u16>().map_err(|_| {
            PaymentError::PaymentProcessingError("Could not parse client id".to_string())
        })?;
        let transaction_id = parts[2].parse::<u32>().map_err(|_| {
            PaymentError::PaymentProcessingError("Could not parse transaction id".to_string())
        })?;
        let amount = parts[3].parse::<f32>().map_err(|_| {
            PaymentError::PaymentProcessingError("Could not parse amount".to_string())
        })?;

        Ok(Self {
            transaction_type,
            client_id,
            transaction_id,
            amount,
            under_dispute: false,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Account {
    client_id: u16,
    available: f32,
    held: f32,
    locked: bool,
    // Map of transaction id to transaction
    transactions: HashMap<u32, Transaction>,
}

impl Account {
    pub fn new(client_id: &u16) -> Self {
        Self {
            client_id: *client_id,
            available: 0.0,
            held: 0.0,
            locked: false,
            transactions: HashMap::new(),
        }
    }

    fn truncate(&self, num: &f32) -> f32 {
        let s = format!("{:.4}", num);
        s.parse::<f32>().expect("truncate failed")
    }

    pub fn total(&self) -> f32 {
        let t = self.held + self.available;
        self.truncate(&t)
    }

    pub fn available(&self) -> f32 {
        self.truncate(&self.available)
    }

    pub fn held(&self) -> f32 {
        self.truncate(&self.held)
    }
}

#[derive(Clone)]
pub struct AccountService {
    // The account service has access to all of the accounts and prevents concurrent
    // payment processors from mutating a single account simultaneously.
    // On the flip side, separate accounts can be worked on concurrently with no issue.

    // HOWEVER, Unfortunately this is not how it works currently... there is an edge case
    // when the account has not been created yet... if all of the accounts would have been
    // previously initialized a Mutex around each account would be sufficient to allow concurrent
    // processing on distinct accounts. I could fix this with more work but don't have time at the moment.

    // Map of client id to account
    accounts: Arc<Mutex<HashMap<u16, Account>>>,
}

impl AccountService {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn process_transaction(&self, transaction: Transaction) -> Result<(), PaymentError> {
        let mut accounts = self.accounts.lock().expect("Ignore lock poisoning");
        let mut account = accounts
            .get(&transaction.client_id)
            .cloned()
            .unwrap_or(Account::new(&transaction.client_id));

        match transaction.transaction_type {
            TransactionType::Deposit => {
                account.available += transaction.amount;
                account
                    .transactions
                    .insert(transaction.transaction_id, transaction.clone());
            }
            TransactionType::Withdrawal => {
                if account.available >= transaction.amount {
                    account.available -= transaction.amount;

                    account
                        .transactions
                        .insert(transaction.transaction_id, transaction.clone());
                }
            }
            TransactionType::Dispute => {
                if let Some(mut disputed_transaction) = account
                    .transactions
                    .get(&transaction.transaction_id)
                    .cloned()
                {
                    // Can only dispute a transaction that isn't already under dispute
                    if !disputed_transaction.under_dispute {
                        account.available -= disputed_transaction.amount;
                        account.held += disputed_transaction.amount;
                        disputed_transaction.under_dispute = true;
                        account
                            .transactions
                            .insert(disputed_transaction.transaction_id, disputed_transaction);
                    }
                }
            }
            TransactionType::Resolve => {
                if let Some(mut disputed_transaction) = account
                    .transactions
                    .get(&transaction.transaction_id)
                    .cloned()
                {
                    // Can only resolve a transaction that is under dispute
                    if disputed_transaction.under_dispute {
                        account.available += disputed_transaction.amount;
                        account.held -= disputed_transaction.amount;
                        disputed_transaction.under_dispute = false;
                        account
                            .transactions
                            .insert(disputed_transaction.transaction_id, disputed_transaction);
                    }
                }
            }
            TransactionType::Chargeback => {
                if let Some(mut disputed_transaction) = account
                    .transactions
                    .get(&transaction.transaction_id)
                    .cloned()
                {
                    // Can only chargeback a transaction that is under dispute
                    if disputed_transaction.under_dispute {
                        account.held -= disputed_transaction.amount;
                        account.locked = true;
                        disputed_transaction.under_dispute = false;
                        account
                            .transactions
                            .insert(disputed_transaction.transaction_id, disputed_transaction);
                    }
                }
            }
        }

        accounts.insert(transaction.client_id, account);
        Ok(())
    }

    pub fn get_account(&self, id: u16) -> Option<Account> {
        let accounts = self.accounts.lock().expect("Ignore lock poisoning");
        return accounts.get(&id).cloned()
    }

    pub fn print_accounts(&self) {
        println!("client,available,held,total,locked");
        let accounts = self.accounts.lock().expect("Ignore lock poisoning");
        for (_, account) in accounts.iter() {
            println!(
                "{},{},{},{},{}",
                account.client_id,
                account.available(),
                account.held(),
                account.total(),
                account.locked
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_service() {
        let account_service = AccountService::new();
        account_service
            .process_transaction(Transaction::from_str("deposit,3,6,37.0").unwrap())
            .await
            .unwrap();
        account_service
            .process_transaction(Transaction::from_str("dispute,3,6,0").unwrap())
            .await
            .unwrap();
        account_service
            .process_transaction(Transaction::from_str("chargeback,3,6,0").unwrap())
            .await
            .unwrap();
        let acct = account_service.get_account(3).unwrap();
        assert_eq!(acct.locked, true);
        assert_eq!(acct.available, 0.0);
        assert_eq!(acct.held, 0.0);
    }

    #[tokio::test]
    async fn test_precision_truncated_at_4() {
        let account_service = AccountService::new();
        account_service
            .process_transaction(Transaction::from_str("deposit,1,1,0.12345").unwrap())
            .await
            .unwrap();
        account_service
            .process_transaction(Transaction::from_str("deposit,2,2,0.12344").unwrap())
            .await
            .unwrap();
        account_service
            .process_transaction(Transaction::from_str("deposit,3,3,0.12").unwrap())
            .await
            .unwrap();
        assert_eq!(account_service.get_account(1).unwrap().total(), 0.1235);
        assert_eq!(account_service.get_account(2).unwrap().total(), 0.1234);
        assert_eq!(account_service.get_account(3).unwrap().total(), 0.12);
    }

    #[tokio::test]
    async fn test_dispute_invalid_tx() {
        let account_service = AccountService::new();
        account_service
            .process_transaction(Transaction::from_str("deposit,1,1,99.0").unwrap())
            .await
            .unwrap();
        assert_eq!(account_service.get_account(1).unwrap().total(), 99.0);
        account_service
            .process_transaction(Transaction::from_str("dispute,1,2,0").unwrap())
            .await
            .unwrap();
        assert_eq!(account_service.get_account(1).unwrap().total(), 99.0);
    }
}
