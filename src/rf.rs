use binread::{BinReaderExt,BinRead, derive_binread, io::{Read, Seek,SeekFrom}};
use modular_bitfield::prelude::*;
use flate2::read::ZlibDecoder;
use std::io::Cursor;
pub use binread::Error;
pub use binread::BinResult;
pub struct RFFile{
    pub header: RFHeader,
    //Add bin read array func
    pub data: Vec<RFData>,
    pub debug_extract: Vec<u8>,
}
impl RFFile{
    pub fn read<R: Read + Seek>(reader: &mut R) -> RFFile{
        let rf_hdr = RFHeader::read(reader).unwrap();
        reader.seek(SeekFrom::Start(rf_hdr.hdr_len.into())).unwrap();
        let mut rf_decomp = Vec::new();
        let mut decomp_zlib = ZlibDecoder::new(reader);
        decomp_zlib.read_to_end(&mut rf_decomp).unwrap();
        let mut rf_de_cursor = Cursor::new(&rf_decomp);
        let mut data_vec = Vec::new();
        for n in 0..rf_hdr.nbr_entrys{
            let mut cur_rfdata = RFData::read(&mut rf_de_cursor).unwrap();
            
            data_vec.push(cur_rfdata);
        }
        RFFile{header:rf_hdr,data:data_vec,debug_extract:rf_decomp}
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
    //Need to vector all our entrys, add in a input so we can know how many, and a string manager due to the crazy index to folder n stuff
    pub offset_in_pack: u32,
    pub name_offset: u32,
    pub cmp_size: u32,
    pub size: u32,
    pub timestamp: u32,
    pub flags: RFFlags,
}

#[bitfield]
#[derive(BinRead,Debug,PartialEq)]
#[br(map = Self::from_bytes)]
pub struct StringInfo {
    pub ext_index : B8,
    pub name_offset : B24,
}

#[bitfield]
#[derive(BinRead,Debug,PartialEq)]
#[br(map = Self::from_bytes)]
pub struct ReltiveStringInfo {
    pub reflen : B5,
    pub refoff : B3,
}

#[bitfield]
#[derive(BinRead,Debug,PartialEq)]
#[br(map = Self::from_bytes)]
pub struct RFFlags {
    folder_depth: B8,
    is_unk0: bool,
    is_folder: bool,
    is_package: bool,
    is_localized: bool,
    is_off_in_pack: bool,
    is_unk1:bool,
    is_overwrite:bool,
    is_unk2:bool,
    extra:u16,
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