use clap::{Arg, ArgAction, Command};
use env_logger::Env;
use log::{error, info};
use shairport_sync_metadata_reader_rs::{Result, ShairportMetadata, ShairportMetadataReader};
use std::process;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let matches = Command::new("shairport-sync-metadata-reader")
        .version("0.1.0")
        .author("Rust Implementation")
        .about("Reads and parses shairport-sync metadata")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Path to metadata file or named pipe")
                .default_value("/tmp/shairport-sync-metadata"),
        )
        .arg(
            Arg::new("continuous")
                .short('c')
                .long("continuous")
                .help("Continuously monitor for new metadata")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("once")
                .short('o')
                .long("once")
                .help("Read metadata once and exit")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stdin")
                .short('s')
                .long("stdin")
                .help("Read from stdin (for piping from shairport-sync)")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let metadata_path = matches.get_one::<String>("path").unwrap().to_string();
    let continuous = matches.get_flag("continuous");
    let once = matches.get_flag("once");
    let stdin_mode = matches.get_flag("stdin");

    let mut reader = ShairportMetadataReader::new(&metadata_path);

    if stdin_mode {
        info!("Reading metadata from stdin");
        if continuous {
            match reader.start_continuous_monitoring().await {
                Ok(mut rx) => {
                    tokio::select! {
                        _ = signal::ctrl_c() => {
                            info!("Received Ctrl+C, shutting down...");
                        }
                        _ = async {
                            while let Some(metadata) = rx.recv().await {
                                print_metadata(&metadata);
                            }
                        } => {}
                    }
                }
                Err(e) => {
                    error!("Failed to start continuous monitoring from stdin: {}", e);
                    process::exit(1);
                }
            }
        } else {
            match reader.read_from_stdin().await {
                Ok(metadata_list) => {
                    for metadata in metadata_list {
                        print_metadata(&metadata);
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    process::exit(1);
                }
            }
        }
    } else if once {
        info!("Reading metadata once from: {}", metadata_path);
        // Default to stdin like C version, unless specific path given and it exists
        let use_stdin = metadata_path == "/tmp/shairport-sync-metadata"
            || !std::path::Path::new(&metadata_path).exists();

        if use_stdin {
            info!("Using stdin for input (like C version)");
            match reader.read_from_stdin().await {
                Ok(metadata_list) => {
                    for metadata in metadata_list {
                        print_metadata(&metadata);
                    }
                }
                Err(e) => {
                    error!("Failed to read from stdin: {}", e);
                    process::exit(1);
                }
            }
        } else {
            match reader.read_metadata_once().await {
                Ok(metadata_list) => {
                    for metadata in metadata_list {
                        print_metadata(&metadata);
                    }
                }
                Err(e) => {
                    error!("Failed to read metadata: {}", e);
                    process::exit(1);
                }
            }
        }
    } else if continuous {
        info!("Starting continuous monitoring of: {}", metadata_path);

        match reader.start_continuous_monitoring_from_file().await {
            Ok(mut rx) => {
                tokio::select! {
                    _ = signal::ctrl_c() => {
                        info!("Received Ctrl+C, shutting down...");
                    }
                    _ = async {
                        while let Some(metadata) = rx.recv().await {
                            print_metadata(&metadata);
                        }
                    } => {}
                }
            }
            Err(e) => {
                error!("Failed to start continuous monitoring: {}", e);
                process::exit(1);
            }
        }
    } else {
        info!("Reading metadata from pipe: {}", metadata_path);
        match reader.read_from_pipe().await {
            Ok(metadata_list) => {
                for metadata in metadata_list {
                    print_metadata(&metadata);
                }
            }
            Err(e) => {
                error!("Failed to read from pipe: {}", e);
                process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_metadata(metadata: &ShairportMetadata) {
    match metadata {
        // Core metadata
        ShairportMetadata::Title(s) => println!("Title: {}", s),
        ShairportMetadata::Artist(s) => println!("Artist: {}", s),
        ShairportMetadata::Album(s) => println!("Album: {}", s),
        ShairportMetadata::Genre(s) => println!("Genre: {}", s),
        ShairportMetadata::Year(s) => println!("Year: {}", s),
        ShairportMetadata::Comment(s) => println!("Comment: {}", s),
        ShairportMetadata::Composer(s) => println!("Composer: {}", s),
        ShairportMetadata::Copyright(s) => println!("Copyright: {}", s),
        ShairportMetadata::TrackNumber(s) => println!("Track Number: {}", s),
        ShairportMetadata::TrackCount(s) => println!("Track Count: {}", s),
        ShairportMetadata::DiscNumber(s) => println!("Disc Number: {}", s),
        ShairportMetadata::DiscCount(s) => println!("Disc Count: {}", s),
        ShairportMetadata::TrackTime(s) => println!("Track Time: {}", s),
        ShairportMetadata::SampleRate(s) => println!("Sample Rate: {}", s),
        ShairportMetadata::ItemId(s) => println!("Item ID: {}", s),
        ShairportMetadata::MediaKind(s) => println!("Media Kind: {}", s),
        ShairportMetadata::DataKind(s) => println!("Data Kind: {}", s),
        ShairportMetadata::PersistentId(s) => println!("Persistent ID: {}", s),
        ShairportMetadata::SortTitle(s) => println!("Sort Title: {}", s),
        ShairportMetadata::SortArtist(s) => println!("Sort Artist: {}", s),
        ShairportMetadata::SortAlbum(s) => println!("Sort Album: {}", s),
        ShairportMetadata::SortComposer(s) => println!("Sort Composer: {}", s),
        ShairportMetadata::UserRating(s) => println!("User Rating: {}", s),
        ShairportMetadata::DataUrl(s) => println!("Data URL: {}", s),
        ShairportMetadata::DateAdded(s) => println!("Date Added: {}", s),
        ShairportMetadata::DateModified(s) => println!("Date Modified: {}", s),
        ShairportMetadata::TimeStamp(s) => println!("Time Stamp: {}", s),
        ShairportMetadata::Kind(s) => println!("Kind: {}", s),

        // SSNC metadata
        ShairportMetadata::PlayBegin => println!("▶️  Play Begin"),
        ShairportMetadata::PlayEnd => println!("⏹️  Play End"),
        ShairportMetadata::PlayFlush => println!("🔄 Play Flush"),
        ShairportMetadata::PlayResume => println!("▶️  Play Resume"),
        ShairportMetadata::PlayVolume(s) => println!("Play Volume: {}", s),
        ShairportMetadata::StreamTitle(s) => println!("Stream Title: {}", s),
        ShairportMetadata::StreamName(s) => println!("Stream Name: {}", s),
        ShairportMetadata::UserAgent(s) => println!("User Agent: {}", s),
        ShairportMetadata::ActiveBegin => println!("🎵 Active Begin"),
        ShairportMetadata::ActiveEnd => println!("⏸️  Active End"),

        // Progress and timing metadata (raw data - format not fully understood)
        ShairportMetadata::Progress(s) => println!("⏱️  Progress: {}", s),
        ShairportMetadata::MetadataStart(s) => println!("📅 Metadata Start: {}", s),
        ShairportMetadata::MetadataEnd(s) => println!("📅 Metadata End: {}", s),

        // Core capabilities and player info
        ShairportMetadata::Capabilities(s) => {
            if s.is_empty() {
                println!("⚙️  Capabilities: (empty)");
            } else {
                println!("⚙️  Capabilities: {}", s);
            }
        }
        ShairportMetadata::MediaPlayer(data) => {
            println!("🎯 Media Player: {} bytes (hex: {:02x?})", data.len(), data);
        }

        // Picture data
        ShairportMetadata::Picture(data) => {
            println!("🖼️  Picture: {} bytes", data.len());
            if data.len() >= 4 {
                let format = match &data[0..4] {
                    [0xFF, 0xD8, 0xFF, _] => "JPEG",
                    [0x89, 0x50, 0x4E, 0x47] => "PNG",
                    _ => "Unknown",
                };
                println!("   Format: {}", format);
            }
        }

        // Other/unknown metadata - show details to help identify missing mappings
        ShairportMetadata::Other {
            item_type,
            code,
            data,
        } => {
            if data.is_empty() {
                println!("🔍 Other [{}:{}]: (no data)", item_type, code);
            } else if let Ok(s) = String::from_utf8(data.clone()) {
                println!("🔍 Other [{}:{}]: {}", item_type, code, s);
            } else if data.len() >= 4 {
                // Check if it might be picture data
                match &data[0..4] {
                    [0xFF, 0xD8, 0xFF, _] => println!(
                        "🔍 Other [{}:{}]: JPEG image, {} bytes",
                        item_type,
                        code,
                        data.len()
                    ),
                    [0x89, 0x50, 0x4E, 0x47] => println!(
                        "🔍 Other [{}:{}]: PNG image, {} bytes",
                        item_type,
                        code,
                        data.len()
                    ),
                    _ => println!(
                        "🔍 Other [{}:{}]: {} bytes (hex: {:02x?}...)",
                        item_type,
                        code,
                        data.len(),
                        &data[..std::cmp::min(8, data.len())]
                    ),
                }
            } else {
                println!(
                    "🔍 Other [{}:{}]: {} bytes (hex: {:02x?})",
                    item_type,
                    code,
                    data.len(),
                    data
                );
            }
        }
    }
}
