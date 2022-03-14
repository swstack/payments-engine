use crate::errors::PaymentError;
use async_trait::async_trait;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

pub enum UriSchemes {
    File,
    S3,
}

impl FromStr for UriSchemes {
    type Err = PaymentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(UriSchemes::File),
            "s3" => Ok(UriSchemes::S3),
            _ => Err(Self::Err::InvalidUriScheme(s.to_string())),
        }
    }
}

#[async_trait]
pub trait Downloadable {
    async fn download(
        &self,
    ) -> Result<Box<dyn Iterator<Item = std::io::Result<String>>>, PaymentError>;
}

pub struct LocalFile {
    pub file_path: String,
}

impl LocalFile {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

pub struct S3File {}

impl S3File {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Downloadable for LocalFile {
    async fn download(
        &self,
    ) -> Result<Box<dyn Iterator<Item = std::io::Result<String>>>, PaymentError> {
        let file = File::open(&self.file_path)
            .map_err(|e| PaymentError::FileDownloadError(e.to_string()))?;
        let lines = BufReader::new(file).lines();
        Ok(Box::new(lines))
    }
}

#[async_trait]
impl Downloadable for S3File {
    async fn download(
        &self,
    ) -> Result<Box<dyn Iterator<Item = std::io::Result<String>>>, PaymentError> {
        unimplemented!()
    }
}
