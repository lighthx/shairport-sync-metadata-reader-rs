# Shairport-Sync Metadata Reader (Rust)

A Rust implementation of a shairport-sync metadata reader that parses metadata into strongly-typed Rust enums.

## Features

- ✅ Parse shairport-sync metadata format (`<item><code><length><data>`)
- ✅ **Strongly-typed Rust enums** for all metadata types
- ✅ Support for all metadata: Core (song info), SSNC (playback control), Picture data
- ✅ Configurable metadata file path
- ✅ Named pipe and file reading
- ✅ Library interface for external projects
- ✅ CLI executable for direct usage
- ✅ Continuous monitoring mode

## Metadata Types (Rust Enums)

The library converts all metadata into a `ShairportMetadata` enum with strongly-typed variants:

### Core Metadata (Song Information)
```rust
ShairportMetadata::Title(String)
ShairportMetadata::Artist(String)
ShairportMetadata::Album(String)
ShairportMetadata::Genre(String)
ShairportMetadata::Year(String)
ShairportMetadata::TrackNumber(String)
// ... and many more
```

### SSNC Metadata (Playback Control)
```rust
ShairportMetadata::PlayBegin
ShairportMetadata::PlayEnd
ShairportMetadata::PlayVolume(String)
ShairportMetadata::StreamTitle(String)
// ... and more
```

### Picture Data
```rust
ShairportMetadata::Picture(Vec<u8>)  // Album artwork
```

### Other/Unknown
```rust
ShairportMetadata::Other { 
    item_type: String, 
    code: String, 
    data: Vec<u8> 
}
```

## Installation

```bash
cargo build --release
```

## Usage

### As a CLI Tool

```bash
# Read from stdin (recommended - pipe from shairport-sync)
shairport-sync --metadata-pipename=/dev/stdout | cargo run -- --stdin

# Continuous monitoring from stdin
shairport-sync --metadata-pipename=/dev/stdout | cargo run -- --stdin --continuous

# Read from named pipe or file
cargo run -- --path /tmp/shairport-sync-metadata

# Read once and exit
cargo run -- --once --path /tmp/metadata
```

### XML Format Support

The implementation correctly parses shairport-sync's XML metadata format:
```xml
<item><type>636f7265</type><code>6d696e6d</code><length>11</length>
<data encoding="base64">
SGVsbG8gV29ybGQ=
</data></item>
```

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
shairport-sync-metadata-reader-rs = { path = "path/to/this/crate" }
tokio = { version = "1.0", features = ["full"] }
```

Example usage with pattern matching:

```rust
use shairport_sync_metadata_reader_rs::{ShairportMetadata, ShairportMetadataReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = ShairportMetadataReader::new("/tmp/shairport-sync-metadata");
    
    // Read metadata once
    let metadata_list = reader.read_metadata_once().await?;
    
    for metadata in metadata_list {
        match metadata {
            ShairportMetadata::Title(title) => {
                println!("Now playing: {}", title);
            },
            ShairportMetadata::Artist(artist) => {
                println!("Artist: {}", artist);
            },
            ShairportMetadata::PlayBegin => {
                println!("Playback started!");
            },
            ShairportMetadata::Picture(data) => {
                println!("Album art: {} bytes", data.len());
                // Save to file, display, etc.
            },
            _ => {} // Handle other variants
        }
    }
    
    // Continuous monitoring
    let mut rx = reader.start_continuous_monitoring().await?;
    while let Some(metadata) = rx.recv().await {
        match metadata {
            ShairportMetadata::Title(title) => println!("♪ {}", title),
            ShairportMetadata::PlayBegin => println!("▶️ Started"),
            ShairportMetadata::PlayEnd => println!("⏹️ Stopped"),
            _ => {}
        }
    }
    
    Ok(())
}
```

## Configuration

The metadata file path can be configured via:
1. Command line argument: `--path /custom/path`
2. Library constructor: `ShairportMetadataReader::new("/custom/path")`
3. Default: `/tmp/shairport-sync-metadata`

## CLI Output Example

```
Title: Shape of You
Artist: Ed Sheeran
Album: ÷ (Deluxe)
▶️  Play Begin
🔊 Volume: -20.0
🖼️  Picture: 45231 bytes
   Format: JPEG
⏹️  Play End
```

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build for release
cargo build --release
```

## Type Safety Benefits

Unlike string-based approaches, this implementation provides:
- **Compile-time safety** - No typos in metadata field names
- **Pattern matching** - Exhaustive handling of all metadata types
- **IDE support** - Autocomplete and IntelliSense for all metadata variants
- **Performance** - No hash map lookups, direct enum matching