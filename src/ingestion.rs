use std::io::BufReader;
use crate::payments::PaymentsQueue;
use crate::errors::PaymentError;
use std::str::FromStr;


enum UriSchemes {
    File,
    S3
}

impl FromStr for UriSchemes {
    type Err = PaymentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file"  => Ok(UriSchemes::File),
            "s3"  => Ok(UriSchemes::S3),
            _      => Err(Self::Err::InvalidUriScheme(s.to_string())),
        }
    }
}

pub struct IngestionService {
    pub payments_queue: PaymentsQueue,
}

impl IngestionService {
    pub fn new(payments_queue: PaymentsQueue) -> Self {
        Self {
            payments_queue
        }
    }

    pub async fn submit_payments(&self, uri: &str) -> Result<(), PaymentError> {
        let uri_parts: Vec<&str> = uri.split("://").collect();
        let path = uri_parts[1];

        let scheme = UriSchemes::from_str(uri_parts[0])?;
        match scheme {
            UriSchemes::File => {
                // TODO
            }
            UriSchemes::S3 => {
                return Err(PaymentError::UriSchemeNotYetSupported);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ingestion::IngestionService;
    use crate::payments::PaymentsQueue;

    #[tokio::test]
    async fn test_submit_payments() {
        let payments_queue = PaymentsQueue::new();
        let ingestion_service = IngestionService::new(payments_queue);
        ingestion_service.submit_payments("file://foo.bar").await;
    }
}
