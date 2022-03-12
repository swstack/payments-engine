pub enum PaymentError {
    InvalidUriScheme(String),
    FileDownloadError(String),
}
