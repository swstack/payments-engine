use crate::errors::PaymentError;
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

pub trait Downloadable {
    fn download(&self) -> Box<dyn Iterator<Item=String>>;
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

impl Downloadable for LocalFile {
    fn download(&self) -> Box<dyn Iterator<Item=String>> {
        todo!()
    }
}

impl Downloadable for S3File {
    fn download(&self) -> Box<dyn Iterator<Item=String>> {
        unimplemented!()
    }
}
