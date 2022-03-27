pub mod engine;

use crate::engine::ingestion::{IngestionService, PaymentsQueue};
use crate::engine::payments::AccountService;

pub fn payments_engine() -> (IngestionService, AccountService) {
    let payments_queue = PaymentsQueue::new();
    let account_service = AccountService::new();
    let ingestion_service =
        IngestionService::new(payments_queue.clone(), account_service.clone(), 1);
    (ingestion_service, account_service)
}
