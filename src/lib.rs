pub mod error;
pub mod metadata;
pub mod parser;
pub mod reader;

pub use error::{MetadataError, Result};
pub use metadata::{MetadataItem, ShairportMetadata};
pub use parser::MetadataParser;
pub use reader::MetadataReader;

pub struct ShairportMetadataReader {
    reader: MetadataReader,
}

impl ShairportMetadataReader {
    pub fn new<P: Into<String>>(metadata_path: P) -> Self {
        Self {
            reader: MetadataReader::new(metadata_path.into()),
        }
    }

    pub async fn read_metadata_once(&mut self) -> Result<Vec<ShairportMetadata>> {
        // Add timeout for reading from potentially blocking sources
        tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            self.reader.read_from_file()
        ).await.unwrap_or_else(|_| {
            log::warn!("Timeout reading metadata, returning what we have");
            Ok(Vec::new())
        })
    }

    pub async fn read_from_pipe(&mut self) -> Result<Vec<ShairportMetadata>> {
        self.reader.read_from_named_pipe().await
    }

    pub async fn read_from_stdin(&mut self) -> Result<Vec<ShairportMetadata>> {
        self.reader.read_from_stdin().await
    }

    pub async fn start_continuous_monitoring(&mut self) -> Result<tokio::sync::mpsc::UnboundedReceiver<ShairportMetadata>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        let mut reader_clone = MetadataReader::new(self.reader.metadata_path.clone());
        tokio::spawn(async move {
            if let Err(e) = reader_clone.start_continuous_reading(tx).await {
                eprintln!("Error in continuous reading: {}", e);
            }
        });
        
        Ok(rx)
    }

    pub async fn start_continuous_monitoring_from_file(&mut self) -> Result<tokio::sync::mpsc::UnboundedReceiver<ShairportMetadata>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        let mut reader_clone = MetadataReader::new(self.reader.metadata_path.clone());
        tokio::spawn(async move {
            if let Err(e) = reader_clone.start_continuous_reading_from_file(tx).await {
                eprintln!("Error in continuous reading: {}", e);
            }
        });
        
        Ok(rx)
    }
}