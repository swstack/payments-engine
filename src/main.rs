mod cli;

use crate::cli::CLI;
use payments_engine::payments_engine;
use std::env;

#[tokio::main]
async fn main() {
    let (ingestion_service, account_service) = payments_engine();
    let cli = CLI::new(ingestion_service.clone());
    let cli_result = cli.execute(env::args().collect()).await;

    if let Some(cli_error) = cli_result.err() {
        panic!("{:?}", cli_error);
    }

    ingestion_service.run().await;
    let results = ingestion_service.shutdown_gracefully().await;
    for result in results {
        if let Some(_error) = result.err() {
            // TODO: If an error happened on one of the workers during
            // payment processing we could log it here
        }
    }

    account_service.print_accounts();
}
