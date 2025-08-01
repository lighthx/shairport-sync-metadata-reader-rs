#[derive(Debug, Clone)]
pub struct MetadataItem {
    pub item_type: String,
    pub code: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum ShairportMetadata {
    // Core metadata (song info)
    Title(String),
    Artist(String),
    Album(String),
    Genre(String),
    Year(String),
    Comment(String),
    Composer(String),
    Copyright(String),
    TrackNumber(String),
    TrackCount(String),
    DiscNumber(String),
    DiscCount(String),
    TrackTime(String),
    SampleRate(String),
    ItemId(String),
    MediaKind(String),
    DataKind(String),
    PersistentId(String),
    SortTitle(String),
    SortArtist(String),
    SortAlbum(String),
    SortComposer(String),
    UserRating(String),
    DataUrl(String),
    DateAdded(String),
    DateModified(String),
    TimeStamp(String),
    Kind(String),

    // SSNC metadata (playback control)
    PlayBegin,
    PlayEnd,
    PlayFlush,
    PlayResume,
    PlayVolume(String),
    StreamTitle(String),
    StreamName(String),
    UserAgent(String),
    ActiveBegin,
    ActiveEnd,
    
    // Progress and timing metadata
    Progress(String),           // ssnc:prgr - playback progress (current/total/end)
    MetadataStart(String),      // ssnc:mdst - metadata start time
    MetadataEnd(String),        // ssnc:mden - metadata end time
    
    // Core capabilities and player info
    Capabilities(String),       // core:caps - device capabilities
    MediaPlayer(Vec<u8>),       // core:mper - media player info

    // Picture data
    Picture(Vec<u8>),

    // Other/unknown metadata
    Other {
        item_type: String,
        code: String,
        data: Vec<u8>,
    },
}

impl ShairportMetadata {
    pub fn from_item(item: &MetadataItem) -> Self {
        // Try to decode data as UTF-8 string, or fall back to hex representation
        let data_str = if item.data.is_empty() {
            String::new()
        } else {
            String::from_utf8(item.data.clone()).unwrap_or_else(|_| {
                // If not valid UTF-8, show as hex
                item.data
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            })
        };

        match (item.item_type.as_str(), item.code.as_str()) {
            // Core metadata mappings
            ("core", "minm") => Self::Title(data_str),
            ("core", "asar") => Self::Artist(data_str),
            ("core", "asal") => Self::Album(data_str),
            ("core", "asgn") => Self::Genre(data_str),
            ("core", "asyr") => Self::Year(data_str),
            ("core", "ascm") => Self::Comment(data_str),
            ("core", "asco") => Self::Composer(data_str),
            ("core", "ascp") => Self::Copyright(data_str),
            ("core", "astn") => Self::TrackNumber(data_str),
            ("core", "astc") => Self::TrackCount(data_str),
            ("core", "asdn") => Self::DiscNumber(data_str),
            ("core", "asdc") => Self::DiscCount(data_str),
            ("core", "asdt") => Self::TrackTime(data_str),
            ("core", "assr") => Self::SampleRate(data_str),
            ("core", "miid") => Self::ItemId(data_str),
            ("core", "mikd") => Self::MediaKind(data_str),
            ("core", "asky") => Self::DataKind(data_str),
            ("core", "aspl") => Self::PersistentId(data_str),
            ("core", "asst") => Self::SortTitle(data_str),
            ("core", "assa") => Self::SortArtist(data_str),
            ("core", "assu") => Self::SortAlbum(data_str),
            ("core", "assc") => Self::SortComposer(data_str),
            ("core", "asur") => Self::UserRating(data_str),
            ("core", "asul") => Self::DataUrl(data_str),
            ("core", "asda") => Self::DateAdded(data_str),
            ("core", "asdm") => Self::DateModified(data_str),
            ("core", "astm") => Self::TimeStamp(data_str),
            ("core", "askd") => Self::Kind(data_str),

            // SSNC metadata mappings
            ("ssnc", "pbeg") => Self::PlayBegin,
            ("ssnc", "pend") => Self::PlayEnd,
            ("ssnc", "pfls") => Self::PlayFlush,
            ("ssnc", "prsm") => Self::PlayResume,
            ("ssnc", "pvol") => Self::PlayVolume(data_str),
            ("ssnc", "stal") => Self::StreamTitle(data_str),
            ("ssnc", "snam") => Self::StreamName(data_str),
            ("ssnc", "snua") => Self::UserAgent(data_str),
            ("ssnc", "abeg") => Self::ActiveBegin,
            ("ssnc", "aend") => Self::ActiveEnd,
            
            // Progress and timing metadata
            ("ssnc", "prgr") => Self::Progress(data_str),
            ("ssnc", "mdst") => Self::MetadataStart(data_str),
            ("ssnc", "mden") => Self::MetadataEnd(data_str),
            
            // Core capabilities and player info
            ("core", "caps") => Self::Capabilities(data_str),
            ("core", "mper") => Self::MediaPlayer(item.data.clone()),

            // Picture data - try multiple possible codes
            ("ssnc", "pict") | ("pict", _) | ("core", "PICT") => Self::Picture(item.data.clone()),

            // Unknown/other metadata
            _ => Self::Other {
                item_type: item.item_type.clone(),
                code: item.code.clone(),
                data: item.data.clone(),
            },
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        match self {
            Self::Title(_) => "Title",
            Self::Artist(_) => "Artist",
            Self::Album(_) => "Album",
            Self::Genre(_) => "Genre",
            Self::Year(_) => "Year",
            Self::Comment(_) => "Comment",
            Self::Composer(_) => "Composer",
            Self::Copyright(_) => "Copyright",
            Self::TrackNumber(_) => "TrackNumber",
            Self::TrackCount(_) => "TrackCount",
            Self::DiscNumber(_) => "DiscNumber",
            Self::DiscCount(_) => "DiscCount",
            Self::TrackTime(_) => "TrackTime",
            Self::SampleRate(_) => "SampleRate",
            Self::ItemId(_) => "ItemId",
            Self::MediaKind(_) => "MediaKind",
            Self::DataKind(_) => "DataKind",
            Self::PersistentId(_) => "PersistentId",
            Self::SortTitle(_) => "SortTitle",
            Self::SortArtist(_) => "SortArtist",
            Self::SortAlbum(_) => "SortAlbum",
            Self::SortComposer(_) => "SortComposer",
            Self::UserRating(_) => "UserRating",
            Self::DataUrl(_) => "DataUrl",
            Self::DateAdded(_) => "DateAdded",
            Self::DateModified(_) => "DateModified",
            Self::TimeStamp(_) => "TimeStamp",
            Self::Kind(_) => "Kind",
            Self::PlayBegin => "PlayBegin",
            Self::PlayEnd => "PlayEnd",
            Self::PlayFlush => "PlayFlush",
            Self::PlayResume => "PlayResume",
            Self::PlayVolume(_) => "PlayVolume",
            Self::StreamTitle(_) => "StreamTitle",
            Self::StreamName(_) => "StreamName",
            Self::UserAgent(_) => "UserAgent",
            Self::ActiveBegin => "ActiveBegin",
            Self::ActiveEnd => "ActiveEnd",
            Self::Progress(_) => "Progress",
            Self::MetadataStart(_) => "MetadataStart", 
            Self::MetadataEnd(_) => "MetadataEnd",
            Self::Capabilities(_) => "Capabilities",
            Self::MediaPlayer(_) => "MediaPlayer",
            Self::Picture(_) => "Picture",
            Self::Other { .. } => "Other",
        }
    }

    pub fn get_data_as_string(&self) -> String {
        match self {
            Self::Title(s) => s.clone(),
            Self::Artist(s) => s.clone(),
            Self::Album(s) => s.clone(),
            Self::Genre(s) => s.clone(),
            Self::Year(s) => s.clone(),
            Self::Comment(s) => s.clone(),
            Self::Composer(s) => s.clone(),
            Self::Copyright(s) => s.clone(),
            Self::TrackNumber(s) => s.clone(),
            Self::TrackCount(s) => s.clone(),
            Self::DiscNumber(s) => s.clone(),
            Self::DiscCount(s) => s.clone(),
            Self::TrackTime(s) => s.clone(),
            Self::SampleRate(s) => s.clone(),
            Self::ItemId(s) => s.clone(),
            Self::MediaKind(s) => s.clone(),
            Self::DataKind(s) => s.clone(),
            Self::PersistentId(s) => s.clone(),
            Self::SortTitle(s) => s.clone(),
            Self::SortArtist(s) => s.clone(),
            Self::SortAlbum(s) => s.clone(),
            Self::SortComposer(s) => s.clone(),
            Self::UserRating(s) => s.clone(),
            Self::DataUrl(s) => s.clone(),
            Self::DateAdded(s) => s.clone(),
            Self::DateModified(s) => s.clone(),
            Self::TimeStamp(s) => s.clone(),
            Self::Kind(s) => s.clone(),
            Self::PlayBegin => String::from("PlayBegin"),
            Self::PlayEnd => String::from("PlayEnd"),
            Self::PlayFlush => String::from("PlayFlush"),
            Self::PlayResume => String::from("PlayResume"),
            Self::PlayVolume(s) => s.clone(),
            Self::StreamTitle(s) => s.clone(),
            Self::StreamName(s) => s.clone(),
            Self::UserAgent(s) => s.clone(),
            Self::ActiveBegin => String::from("ActiveBegin"),
            Self::ActiveEnd => String::from("ActiveEnd"),
            Self::Progress(s) => s.clone(),
            Self::MetadataStart(s) => s.clone(),
            Self::MetadataEnd(s) => s.clone(),
            Self::Capabilities(s) => s.clone(),
            Self::MediaPlayer(_) => String::from("MediaPlayer"),
            Self::Picture(_) => String::from("Picture"),
            Self::Other { .. } => String::from("Other"),
        }
    }
}
