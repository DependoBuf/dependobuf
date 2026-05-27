//! Module exports `FileContent` struct with methods to read file on disk.
use std::fs;
use std::path::PathBuf;

use thiserror::Error;

pub struct FileContent {
    /// Name of file.
    name: String,
    /// Content of file.
    content: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("'{0}' is not a valid file name")]
    NotFile(PathBuf),
    #[error("Can't read file '{0}': {1}")]
    NoOpen(PathBuf, std::io::Error),
    #[error("File extension is not .dbuf")]
    NoDbuf(PathBuf),
}

impl FileContent {
    /// Read file and create File struct.
    pub fn new(file: &PathBuf) -> Result<FileContent, Error> {
        let file_name = file
            .file_stem()
            .ok_or_else(|| Error::NotFile(file.to_owned()))?;
        let file_ext = file
            .extension()
            .ok_or_else(|| Error::NotFile(file.to_owned()))?;

        if file_ext != "dbuf" {
            return Err(Error::NoDbuf(file.clone()));
        }

        let content = fs::read_to_string(file).map_err(|e| Error::NoOpen(file.clone(), e))?;

        Ok(FileContent {
            name: file_name.to_string_lossy().into(),
            content,
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}
