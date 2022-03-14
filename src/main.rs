mod cli;
mod download;
mod errors;
mod ingestion;
mod payments;

use crate::cli::CLI;
use crate::ingestion::{IngestionService, PaymentsQueue};
use crate::payments::PaymentsProcessor;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let payments_queue = PaymentsQueue::new();

    let mut tasks = Vec::new();
    for _ in 0..3 {
        let queue = payments_queue.clone();
        tasks.push(tokio::spawn(async move {
            PaymentsProcessor::new(queue).start().await;
        }));
    }

    let ingestion_service = IngestionService::new(payments_queue.clone());
    let cli = CLI::new(ingestion_service);
    let result = cli.execute(env::args().collect()).await;

    if let Some(cli_error) = result.err() {
        panic!("{:?}", cli_error);
    }

    for t in tasks {
        t.await;
    }
}
