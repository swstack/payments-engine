mod cli;
mod download;
mod errors;
mod ingestion;
mod payments;

use crate::cli::CLI;
use crate::ingestion::IngestionService;
use crate::payments::{PaymentsProcessor, PaymentsQueue};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let payments_queue = PaymentsQueue::new();

    let mut tasks = Vec::new();
    for i in 0..2 {
        let queue = payments_queue.clone();
        tasks.push(tokio::spawn(async move {
            PaymentsProcessor::new(format!("p{:?}", i), queue)
                .start()
                .await;
        }));
    }

    let ingestion_service = IngestionService::new(payments_queue.clone());
    let cli = CLI::new(ingestion_service);
    cli.execute(env::args().collect());

    for t in tasks {
        t.await;
    }
    println!("Finished");
}
