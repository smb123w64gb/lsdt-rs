use binrw::{binread,Error,BinResult, BinReaderExt,BinRead, NullString, io::{Read,Cursor, Seek,SeekFrom}};
use modular_bitfield::prelude::*;
use flate2::read::ZlibDecoder;
pub struct RFInfo{
    pub is_compressed : bool,
    pub is_folder : bool,
    pub folder_depth: u32,
    pub file_offset : u32,
    pub file_size : u32,
    pub file_name : String,

}
pub struct RFFile{
    pub debug_extract: Vec<u8>,
    pub entrys: Vec<RFInfo>,
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
            let mut cur_rfdata = RFEntry::read(&mut rf_de_cursor).unwrap();
            
            data_vec.push(cur_rfdata);
        }
        rf_de_cursor.seek(SeekFrom::Start((rf_hdr.offset_names - rf_hdr.hdr_len).into())).unwrap();
        
        let rfstrings : Vec<u8> = RFStr::read(&mut rf_de_cursor).unwrap().strbin.into();
        let mut string_cursor = Cursor::new(&rfstrings);
        let rfexts:Vec<u32> = RFExt::read(&mut rf_de_cursor).unwrap().exts.into();
        let mut extention:Vec<String> = Vec::new();
        for n in rfexts.iter(){
            string_cursor.seek(SeekFrom::Start(*n as u64)).unwrap();
            let teststring : NullString = string_cursor.read_le().unwrap();
            extention.push(teststring.into_string());
        }
        let mut all_strings : Vec<String> = Vec::new();
        for n in data_vec.iter(){
            string_cursor.seek(SeekFrom::Start(n.name_info.name_offset() as u64)).unwrap();
            if(n.name_info.ext_data() == 1){
                let info : ReltiveStringInfo = string_cursor.read_le().unwrap();
                let mid : NullString = string_cursor.read_le().unwrap();
                //println!("{0}",(((info.refoffhi() as u32) << 7)  as u32));
                string_cursor.seek(SeekFrom::Start((n.name_info.name_offset() - (((info.refoffhi() as u32) << 7) + info.refofflw() as u32))  as u64)).unwrap();
                let lowstr : NullString = string_cursor.read_le().unwrap();
                let mut low = lowstr.into_string().chars().collect::<Vec<char>>();
                low.truncate((info.reflen() + 4)  as usize);
                let lowstr : String = low.into_iter().collect();
                all_strings.push(String::from(format!("{0}{1}{2}",lowstr,mid.into_string(),extention[n.name_info.ext_index() as usize])));

            }
            else{
                let allnows : NullString = string_cursor.read_le().unwrap();
                all_strings.push(String::from(format!("{0}{1}",allnows.into_string(),extention[n.name_info.ext_index() as usize])));
            }
            
        }
        let mut allinfo : Vec<RFInfo> = Vec::new();
        for n in 0..rf_hdr.nbr_entrys{
            allinfo.push(RFInfo{is_compressed : *&data_vec[n as usize].flags.is_package(),
                is_folder : *&data_vec[n as usize].flags.is_folder(),
                folder_depth : *&data_vec[n as usize].folder_depth.into(),
                file_offset : *&data_vec[n as usize].offset_in_pack,
                file_size : *&data_vec[n as usize].size,
                file_name : all_strings[n as usize].to_owned()})
        }
        //let rfexts = Vec::new();
        RFFile{debug_extract:rf_decomp,entrys:allinfo}
    } 

}

#[derive(BinRead)]
pub struct RFHeader {
    pub magic: u16,
    pub ver: u16,
    pub hdr_len: u32,
    pub padding: u32,
    pub resource_entry: u32,
    
    pub resource_size: u32,
    pub timestamp: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,

    pub offset_names: u32,
    pub name_size: u32,
    pub nbr_entrys: u32,
}
#[derive(BinRead)]
pub struct RFEntry {
    //Need to vector all our entrys, add in a input so we can know how many, and a string manager due to the crazy index to folder n stuff
    pub offset_in_pack: u32,
    pub name_info: StringInfo,
    pub cmp_size: u32,
    pub size: u32,
    pub timestamp: u32,
    pub folder_depth: u8,
    pub flags: RFFlags,
    pub padding :u16,
}

#[derive(BinRead)]
pub struct RFStr {
    pub cnt: u32,
    #[br(args{count : (cnt * 0x2000).try_into().unwrap()})]
    pub strbin: Vec<u8>,
}

#[derive(BinRead)]
pub struct RFExt {
    pub count: u32,
    #[br(args{count : count as usize})]
    pub exts: Vec<u32>,
}

#[bitfield]
#[derive(BinRead,Debug,PartialEq)]
#[br(map = Self::from_bytes)]
pub struct StringInfo {
    pub name_offset : B23,
    pub ext_data : B1,
    pub ext_index : B8,
}

#[bitfield]
#[derive(BinRead,Debug,PartialEq)]
#[br(map = Self::from_bytes)]
pub struct ReltiveStringInfo {
    pub reflen : B5,
    pub refoffhi : B3,
    pub refofflw : B8,
}

#[bitfield]
#[derive(BinRead,Debug,PartialEq)]
#[br(map = Self::from_bytes)]
pub struct RFFlags {
    is_unk0: bool,
    is_folder: bool,
    is_package: bool,
    is_localized: bool,
    is_off_in_pack: bool,
    is_unk1:bool,
    is_overwrite:bool,
    is_unk2:bool,
}

impl RFHeader{
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}
impl RFEntry{
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}
impl RFStr{
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}
impl RFExt{
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}