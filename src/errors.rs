pub enum PaymentError {
    InvalidUriScheme(String),
    UriSchemeNotYetSupported,
    FileDownloadError(String),
}