use crate::engine::download::{Downloadable, LocalFile, S3File, UriSchemes};
use crate::engine::errors::PaymentError;
use crate::engine::payments::AccountService;
use crate::engine::payments::PaymentsProcessor;
use std::collections::vec_deque::VecDeque;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

pub struct IngestionServiceInner {
    pub payments_queue: PaymentsQueue,
    pub account_service: AccountService,
    pub num_workers: u8,
    pub workers: Vec<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct IngestionService {
    pub payments_queue: PaymentsQueue,
    pub account_service: AccountService,
    pub num_workers: u8,
    pub workers: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl IngestionService {
    pub fn new(
        payments_queue: PaymentsQueue,
        account_service: AccountService,
        num_workers: u8,
    ) -> Self {
        Self {
            payments_queue,
            account_service,
            num_workers,
            workers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run(&self) {
        for _ in 0..self.num_workers {
            let payments_queue_clone = self.payments_queue.clone();
            let account_service_clone = self.account_service.clone();

            let worker = tokio::spawn(async move {
                let processing_result =
                    PaymentsProcessor::new(payments_queue_clone, account_service_clone)
                        .start()
                        .await;
                if let Some(processing_error) = processing_result.err() {
                    panic!("{:?}", processing_error);
                }
            });
            self.workers.lock().expect("").push(worker);
        }
    }

    pub async fn shutdown_gracefully(&self) {
        for worker in self
            .workers
            .lock()
            .expect("Ignore lock poisoning")
            .iter_mut()
        {
            let processing_result = worker.await;
            if let Some(processing_error) = processing_result.err() {
                panic!("{:?}", processing_error);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_ingestion_service() {
        let payments_queue = PaymentsQueue::new();
        let ingestion_service = IngestionService::new(payments_queue.clone());

        let out = "skip\nfoo\nbar\nbaz";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(out.as_bytes()).unwrap();

        let file_path = file.path().to_str().unwrap();
        let uri = format!("file://{}", file_path);
        ingestion_service.submit_payments_csv(&uri).await.unwrap();

        assert_eq!(payments_queue.get_transaction().unwrap(), "foo");
        assert_eq!(payments_queue.get_transaction().unwrap(), "bar");
        assert_eq!(payments_queue.get_transaction().unwrap(), "baz");
    }
}
