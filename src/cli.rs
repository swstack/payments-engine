use crate::IngestionService;

pub struct CLI {
    ingestion_service: IngestionService,
}

impl CLI {
    pub fn new(ingestion_service: IngestionService) -> Self {
        Self { ingestion_service }
    }

    pub fn execute(&self, args: Vec<String>) {}
}
