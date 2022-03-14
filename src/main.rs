mod cli;
mod download;
mod errors;
mod ingestion;
mod payments;

use crate::cli::CLI;
use crate::ingestion::{IngestionService, PaymentsQueue};
use crate::payments::{AccountService, PaymentsProcessor};
use std::env;

#[tokio::main]
async fn main() {
    let payments_queue = PaymentsQueue::new();
    let account_service = AccountService::new();

    let ingestion_service = IngestionService::new(payments_queue.clone());
    let cli = CLI::new(ingestion_service);
    let cli_result = cli.execute(env::args().collect()).await;

    if let Some(cli_error) = cli_result.err() {
        panic!("{:?}", cli_error);
    }

    // Increase workers for more POWER (keeping at 1 for now for debuggability and lack of testing)
    let num_workers = 1;
    let mut tasks = Vec::new();
    for _ in 0..num_workers {
        let payments_queue_clone = payments_queue.clone();
        let account_service_clone = account_service.clone();
        tasks.push(tokio::spawn(async move {
            let processing_result =
                PaymentsProcessor::new(payments_queue_clone, account_service_clone)
                    .start()
                    .await;
            if let Some(processing_error) = processing_result.err() {
                panic!("{:?}", processing_error);
            }
        }));
    }

    for t in tasks {
        let processing_result = t.await;
        if let Some(processing_error) = processing_result.err() {
            panic!("{:?}", processing_error);
        }
    }

    account_service.print_accounts();
}
