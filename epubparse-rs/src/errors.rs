use std::{io, string};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("File error")]
    FileError(#[from] io::Error),
    #[error("Could not unzip the epub file")]
    ZipError(#[from] zip::result::ZipError),
    #[error("Invalid UTF8")]
    UTF8Error(#[from] string::FromUtf8Error),
    #[error(transparent)]
    EpubError(#[from] MalformattedEpubError),
}

#[derive(Error, Debug)]
pub enum MalformattedEpubError {
    #[error("Malformatted/missing container.xml file")]
    MalformattedContainer,
    #[error("Malformatted content.opf file")]
    MalformattedContentOpf,
    #[error("Malformatted toc.ncx file")]
    MalformattedTocNcx,
}
