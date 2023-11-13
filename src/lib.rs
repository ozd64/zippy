use archive::ReadableArchive;

pub mod archive;
pub mod date_time;
pub mod headers;
pub mod pretty_printer;
pub mod zip;
pub mod zip_crypto;

pub type RefReadableArchive = Box<dyn ReadableArchive>;
pub type Crc32 = u32;
