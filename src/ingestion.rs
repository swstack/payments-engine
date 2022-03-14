use crate::download::{Downloadable, LocalFile, S3File, UriSchemes};
use crate::errors::PaymentError;
use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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

        for payment_string in downloadable.download().await?.skip(1) {
            self.payments_queue.publish_transaction(payment_string?);
        }

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

    pub fn publish_transaction(&self, message: String) {
        self.queue
            .lock()
            .expect("Ignore lock poisoning")
            .push_back(message);
    }

    pub fn get_transaction(&self) -> Option<String> {
        self.queue
            .lock()
            .expect("Ignore lock poisoning")
            .pop_front()
    }
}
