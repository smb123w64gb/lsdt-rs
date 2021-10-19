use binread::{BinReaderExt, derive_binread, io::{Read, Seek,SeekFrom}};
use flate2::read::ZlibDecoder;
use std::io::Cursor;
pub use binread::Error;
pub use binread::BinResult;
pub struct RFFile{
    pub header: RFHeader,
    pub data: RFData,
}
impl RFFile{
    pub fn read<R: Read + Seek>(reader: &mut R) -> RFFile{
        let rf_hdr = RFHeader::read(reader).unwrap();
        reader.seek(SeekFrom::Start(rf_hdr.hdr_len.into())).unwrap();
        let mut rf_decomp = Vec::new();
        let mut decomp_zlib = ZlibDecoder::new(reader);
        decomp_zlib.read_to_end(&mut rf_decomp).unwrap();
        let mut rf_de_cursor = Cursor::new(rf_decomp);
        RFFile{header:rf_hdr,data:RFData::read(&mut rf_de_cursor).unwrap()}
    } 
}

#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct RFHeader {
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
#[derive_binread]
#[derive(Debug, PartialEq)]
pub struct RFData {
    pub magic: u32,
    pub ver: u32,
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
impl RFHeader{
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}
impl RFData{
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}