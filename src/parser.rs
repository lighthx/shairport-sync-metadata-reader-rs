use crate::error::{MetadataError, Result};
use crate::metadata::{MetadataItem, ShairportMetadata};
use std::convert::TryInto;
use std::collections::VecDeque;
use log::debug;
use base64::{engine::general_purpose, Engine as _};

pub struct MetadataParser {
    buffer: Vec<u8>,
    position: usize,
}

impl MetadataParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }

    pub fn feed_data(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        debug!("Fed {} bytes, total buffer size: {}", data.len(), self.buffer.len());
    }

    pub fn parse_next_item(&mut self) -> Result<Option<MetadataItem>> {
        // Need at least 12 bytes for: item_type(4) + code(4) + length(4)
        if self.buffer.len() - self.position < 12 {
            debug!("Not enough bytes for header: need 12, have {}", self.buffer.len() - self.position);
            return Ok(None);
        }

        let item_type = String::from_utf8(
            self.buffer[self.position..self.position + 4].to_vec()
        ).unwrap_or_else(|_| {
            format!("0x{:02x}{:02x}{:02x}{:02x}", 
                self.buffer[self.position], self.buffer[self.position + 1],
                self.buffer[self.position + 2], self.buffer[self.position + 3])
        });
        self.position += 4;

        let code = String::from_utf8(
            self.buffer[self.position..self.position + 4].to_vec()
        ).unwrap_or_else(|_| {
            format!("0x{:02x}{:02x}{:02x}{:02x}", 
                self.buffer[self.position], self.buffer[self.position + 1],
                self.buffer[self.position + 2], self.buffer[self.position + 3])
        });
        self.position += 4;

        let length_bytes: [u8; 4] = self.buffer[self.position..self.position + 4]
            .try_into()
            .map_err(|_| MetadataError::InvalidFormat)?;
        let length = u32::from_be_bytes(length_bytes) as usize;
        self.position += 4;

        debug!("Parsing item: type='{}', code='{}', length={}", item_type, code, length);

        // Check if we have enough data for the payload
        if self.buffer.len() - self.position < length {
            debug!("Not enough bytes for payload: need {}, have {}", length, self.buffer.len() - self.position);
            // Reset position to try again later
            self.position -= 12;
            return Ok(None);
        }

        let data = self.buffer[self.position..self.position + length].to_vec();
        self.position += length;

        debug!("Successfully parsed item with {} bytes of data", data.len());

        Ok(Some(MetadataItem {
            item_type,
            code,
            data,
        }))
    }

    pub fn parse_next_metadata(&mut self) -> Result<Option<ShairportMetadata>> {
        if let Some(item) = self.parse_next_item()? {
            Ok(Some(ShairportMetadata::from_item(&item)))
        } else {
            Ok(None)
        }
    }

    pub fn clear_processed(&mut self) {
        if self.position > 0 {
            debug!("Clearing {} processed bytes from buffer", self.position);
            self.buffer.drain(0..self.position);
            self.position = 0;
        }
    }
}

impl Default for MetadataParser {
    fn default() -> Self {
        Self::new()
    }
}

// XML parser for stdin input (like C version)
pub struct XmlMetadataParser {
    buffer: String,
    pending_lines: VecDeque<String>,
}

impl XmlMetadataParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            pending_lines: VecDeque::new(),
        }
    }

    pub fn feed_data(&mut self, data: &[u8]) {
        if let Ok(text) = std::str::from_utf8(data) {
            self.buffer.push_str(text);
            
            // Split into lines and add to pending queue
            while let Some(newline_pos) = self.buffer.find('\n') {
                let line = self.buffer[..newline_pos].to_string();
                self.buffer.drain(..=newline_pos);
                
                if !line.trim().is_empty() {
                    self.pending_lines.push_back(line);
                }
            }
        }
    }

    pub fn parse_next_metadata(&mut self) -> Result<Option<ShairportMetadata>> {
        while let Some(line) = self.pending_lines.pop_front() {
            
            // Parse C-style format: <item><type>636f7265</type><code>6d696e6d</code><length>11</length>
            if let Some(captures) = regex::Regex::new(
                r"<item><type>([0-9a-fA-F]+)</type><code>([0-9a-fA-F]+)</code><length>(\d+)</length>"
            ).unwrap().captures(&line) {
                let type_hex = &captures[1];
                let code_hex = &captures[2];
                let length: usize = captures[3].parse().map_err(|_| MetadataError::InvalidFormat)?;
                
                // Convert hex strings to 4-character strings
                let type_num = u32::from_str_radix(type_hex, 16)
                    .map_err(|_| MetadataError::InvalidFormat)?;
                let code_num = u32::from_str_radix(code_hex, 16)
                    .map_err(|_| MetadataError::InvalidFormat)?;
                
                let item_type = String::from_utf8(type_num.to_be_bytes().to_vec())
                    .unwrap_or_else(|_| format!("0x{:08x}", type_num));
                let code = String::from_utf8(code_num.to_be_bytes().to_vec())
                    .unwrap_or_else(|_| format!("0x{:08x}", code_num));
                

                let mut data = Vec::new();
                
                if length > 0 {
                    // We need at least 2 more lines: data start, base64 data+end combined
                    if self.pending_lines.len() < 2 {
                        // Put the line back and wait for more data
                        self.pending_lines.push_front(line);
                        return Ok(None);
                    }
                    
                    // Look for data start tag
                    if let Some(data_start_line) = self.pending_lines.pop_front() {
                        if data_start_line.trim() == r#"<data encoding="base64">"# {
                            // Get the base64 data line (which includes </data></item>)
                            if let Some(base64_line) = self.pending_lines.pop_front() {
                                // Extract base64 data from line like "U2F5IFlvdSwgU2F5IE1lIOKAlCBPY2VhbnNpZGUvU2VhbGlmZS9CZWFjaGxpZmU=</data></item>"
                                if let Some(end_pos) = base64_line.find("</data>") {
                                    let base64_data = &base64_line[..end_pos];
                                    
                                    // Decode base64
                                    if let Ok(decoded) = general_purpose::STANDARD.decode(base64_data.trim()) {
                                        data = decoded;
                                    }
                                }
                            }
                        }
                    }
                }

                let item = MetadataItem {
                    item_type: item_type.clone(),
                    code: code.clone(),
                    data: data.clone(),
                };
                
                return Ok(Some(ShairportMetadata::from_item(&item)));
            }
        }
        
        Ok(None)
    }

    pub fn clear_processed(&mut self) {
        // Keep some buffer for incomplete lines
        if self.buffer.len() > 10000 {
            self.buffer.clear();
        }
    }
}

impl Default for XmlMetadataParser {
    fn default() -> Self {
        Self::new()
    }
}