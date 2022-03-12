use std::collections::VecDeque;
use std::sync::Mutex;

pub struct PaymentsQueue {
    queue: Mutex<VecDeque<String>>,
}

impl PaymentsQueue {
    pub fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }
}

pub struct PaymentsProcessor {}

impl PaymentsProcessor {
    pub fn new() -> Self {
        Self {}
    }
}
