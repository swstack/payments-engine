use std::io::Error;

#[derive(Clone, Debug)]
pub enum PaymentError {
    InvalidUriScheme(String),
    FileDownloadError(String),
    PaymentProcessingError(String),
    CliError(String),
}

impl From<std::io::Error> for PaymentError {
    fn from(e: Error) -> Self {
        PaymentError::FileDownloadError(e.to_string())
    }
}
