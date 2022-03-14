use crate::errors::PaymentError;
use crate::ingestion::PaymentsQueue;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct PaymentsProcessor {
    payments_queue: PaymentsQueue,
}

impl PaymentsProcessor {
    pub fn new(payments_queue: PaymentsQueue) -> Self {
        Self { payments_queue }
    }

    pub async fn start(&self) {
        loop {
            let payment_string = self.payments_queue.get_transaction().await;
        }
    }
}
