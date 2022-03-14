use crate::download::{Downloadable, LocalFile, S3File, UriSchemes};
use crate::errors::PaymentError;
use crate::payments::PaymentsQueue;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct IngestionService {
    pub payments_queue: PaymentsQueue,
}

impl IngestionService {
    pub fn new(payments_queue: PaymentsQueue) -> Self {
        Self { payments_queue }
    }

    pub async fn submit_payments_csv(&self, uri: &str) -> Result<(), PaymentError> {
        let uri_parts: Vec<&str> = uri.split("://").collect();
        let scheme = UriSchemes::from_str(uri_parts[0])?;
        let path = uri_parts[1];

        let downloadable: Box<dyn Downloadable> = match scheme {
            UriSchemes::File => Box::new(LocalFile::new(path)),
            UriSchemes::S3 => Box::new(S3File::new()),
        };

        for payment_string in downloadable.download() {}

        Ok(())
    }
}

#[derive(Clone)]
pub struct PaymentsQueue {
    queue: Arc<Mutex<VecDeque<String>>>,
}

impl PaymentsQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn get_payment_request(&self) -> Option<String> {
        // Attempt 3 times to get something from the queue, waiting 1 each time
        for _ in 0..2 {
            let message = self
                .queue
                .lock()
                .expect("Ignore lock poisoning")
                .pop_front();

            if message.is_some() {
                return message;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        None
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::ingestion::IngestionService;
//     use crate::payments::PaymentsQueue;
//
//     #[tokio::test]
//     async fn test_submit_payments() {
//         let payments_queue = PaymentsQueue::new();
//         let ingestion_service = IngestionService::new(payments_queue);
//         ingestion_service
//             .submit_payments_csv("file://foo.csv")
//             .await;
//     }
// }
