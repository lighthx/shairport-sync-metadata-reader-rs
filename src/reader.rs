use crate::error::{MetadataError, Result};
use crate::metadata::ShairportMetadata;
use log::{debug, error, info, warn};
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader, stdin};
use tokio::sync::mpsc;

pub struct MetadataReader {
    pub metadata_path: String,
}

impl MetadataReader {
    pub fn new(metadata_path: String) -> Self {
        Self {
            metadata_path,
        }
    }

    pub async fn read_from_file(&mut self) -> Result<Vec<ShairportMetadata>> {
        let path = Path::new(&self.metadata_path);
        info!("Reading metadata from file: {}", path.display());

        // Use XML parser for file reading since shairport-sync outputs XML format
        let mut xml_parser = crate::parser::XmlMetadataParser::new();
        let file = OpenOptions::new().read(true).open(path).await?;
        let mut reader = BufReader::new(file);
        let mut metadata_items = Vec::new();
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("Reached EOF on file");
                    break;
                }
                Ok(_) => {
                    xml_parser.feed_data(line.as_bytes());
                    
                    while let Some(metadata) = xml_parser.parse_next_metadata()? {
                        metadata_items.push(metadata);
                    }
                    
                    xml_parser.clear_processed();
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        continue;
                    }
                    return Err(MetadataError::Io(e));
                }
            }
        }

        info!("Finished reading. Total metadata items: {}", metadata_items.len());
        Ok(metadata_items)
    }

    pub async fn read_from_named_pipe(&mut self) -> Result<Vec<ShairportMetadata>> {
        let path = Path::new(&self.metadata_path);
        info!("Reading metadata from named pipe: {}", path.display());

        // Use XML parser for named pipe reading since shairport-sync outputs XML format
        let mut xml_parser = crate::parser::XmlMetadataParser::new();
        let file = OpenOptions::new().read(true).open(path).await?;
        let mut reader = BufReader::new(file);
        let mut metadata_items = Vec::new();
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("EOF on named pipe");
                    break;
                }
                Ok(_) => {
                    xml_parser.feed_data(line.as_bytes());

                    while let Some(metadata) = xml_parser.parse_next_metadata()? {
                        metadata_items.push(metadata);
                    }

                    xml_parser.clear_processed();
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        continue;
                    }
                    return Err(MetadataError::Io(e));
                }
            }
        }

        Ok(metadata_items)
    }

    pub async fn read_from_stdin(&mut self) -> Result<Vec<ShairportMetadata>> {
        info!("Reading metadata from stdin (XML format detected)");

        // Use XML parser for stdin since data is in XML format
        let mut xml_parser = crate::parser::XmlMetadataParser::new();

        let stdin = stdin();
        let mut reader = BufReader::new(stdin);
        let mut metadata_items = Vec::new();
        let mut line = String::new();

        info!("Starting stdin XML read loop...");
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("Reached EOF on stdin");
                    break; // EOF
                }
                Ok(_) => {
                    debug!("Read XML line from stdin: {}", line.trim());
                    xml_parser.feed_data(line.as_bytes());

                    while let Some(metadata) = xml_parser.parse_next_metadata()? {
                        metadata_items.push(metadata);
                    }

                    xml_parser.clear_processed();
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    return Err(MetadataError::Io(e));
                }
            }
        }

        info!(
            "Finished reading from stdin. Total metadata items: {}",
            metadata_items.len()
        );
        Ok(metadata_items)
    }

    pub async fn start_continuous_reading(
        &mut self,
        tx: mpsc::UnboundedSender<ShairportMetadata>,
    ) -> Result<()> {
        info!("Starting continuous reading from stdin");

        // Use XML parser for stdin reading since shairport-sync outputs XML format
        let mut xml_parser = crate::parser::XmlMetadataParser::new();
        let stdin = stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    xml_parser.feed_data(line.as_bytes());

                    while let Some(metadata) = xml_parser.parse_next_metadata()? {
                        if tx.send(metadata).is_err() {
                            info!("Receiver closed, stopping continuous reading");
                            return Ok(());
                        }
                    }

                    xml_parser.clear_processed();
                }
                Err(e) => {
                    warn!("Error in continuous reading: {}", e);
                    return Err(MetadataError::Io(e));
                }
            }
        }

        Ok(())
    }

    pub async fn start_continuous_reading_from_file(
        &mut self,
        tx: mpsc::UnboundedSender<ShairportMetadata>,
    ) -> Result<()> {
        let path = Path::new(&self.metadata_path);
        info!("Starting continuous reading from: {}", path.display());

        // Use XML parser for file reading since shairport-sync outputs XML format
        let mut xml_parser = crate::parser::XmlMetadataParser::new();
        
        // Use OpenOptions for better named pipe handling
        let file = OpenOptions::new().read(true).open(path).await?;

        let mut reader = BufReader::new(file);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("No data available, waiting...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                Ok(_) => {
                    debug!("Read line from file: {}", line.trim());
                    xml_parser.feed_data(line.as_bytes());

                    while let Some(metadata) = xml_parser.parse_next_metadata()? {
                        debug!("Parsed metadata: {}", metadata.get_type_name());
                        if tx.send(metadata).is_err() {
                            info!("Receiver closed, stopping continuous reading");
                            return Ok(());
                        }
                    }

                    xml_parser.clear_processed();
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        debug!("Would block on continuous read, waiting...");
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        continue;
                    }
                    warn!("Error in continuous reading: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}
