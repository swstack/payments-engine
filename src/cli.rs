use payments_engine::engine::errors::PaymentError;
use payments_engine::engine::ingestion::IngestionService;

pub struct CLI {
    ingestion_service: IngestionService,
}

impl CLI {
    pub fn new(ingestion_service: IngestionService) -> Self {
        Self { ingestion_service }
    }

    pub async fn execute(&self, args: Vec<String>) -> Result<(), PaymentError> {
        // Discard first arg which is the cwd
        // Assume only 1 positional arg which is a file path
        if args.len() < 2 {
            return Err(PaymentError::CliError("Provide input file".to_string()));
        }
        self.ingestion_service
            .submit_payments_csv(&format!("file://{}", args[1]))
            .await
    }
}
