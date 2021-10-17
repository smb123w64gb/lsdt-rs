use binread::{BinReaderExt, derive_binread, io::{Read, Seek}};
use std::io::BufReader;
use std::path::Path;

pub use binread::Error;
pub use binread::BinResult;

mod ls_str;

#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct RFFile {
    pub magic: u16,
    pub ver: u16,
    pub hdr_len: u32,
    pub padding: u32,
    pub resource_entry: u32,
    
    #[br(temp)]
    pub resource_size: u32,
    pub timestamp: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,

    pub offset_names: u32,
    pub name_size: u32,
    pub nbr_entrys: u32,

   
}
impl RFFile{
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        BufReader::new(std::fs::File::open(path)?).read_le()
    }
    #[allow(unused)]
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}