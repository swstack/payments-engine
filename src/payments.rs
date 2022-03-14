use crate::errors::PaymentError;
use crate::ingestion::PaymentsQueue;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct PaymentsProcessor {
    // Name only used for debugging
    name: String,
    payments_queue: PaymentsQueue,
}

impl PaymentsProcessor {
    pub fn new(name: String, payments_queue: PaymentsQueue) -> Self {
        Self {
            name,
            payments_queue,
        }
    }

    pub async fn start(&self) {
        loop {
            println!(
                "Processor: {:?},  thread: {:?} waiting for payment...",
                self.name,
                thread::current().name()
            );
            let payment_string = self.payments_queue.get_payment_request().await;
        }
    }
}
