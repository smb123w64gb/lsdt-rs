use binrw::{BinReaderExt, BinRead, BinResult};
use std::io::BufReader;
use std::path::Path;


pub mod ls_str;
#[derive(BinRead)]
pub struct LSFile {
    pub magic: u16,
    pub ver: u16,
    
    file_count: u32,

    #[br(args{count: file_count as usize, inner:(ver,)})]
    pub ls_entry: Vec<LSEntry>,

   
}
#[derive(BinRead,Copy,Clone)]
#[br(import(ver: u16))]
#[derive(Default)]
pub struct LSEntry {

    pub crc: u32,
    pub offset: u32,
    pub size: u32,
    #[br(if(ver == 2 ))]
    pub dt_index: Option<u16>,
    #[br(if(ver == 2 ))]
    pub padding: Option<u16>,
    
}

impl LSFile{
    pub fn open<P: AsRef<Path>>(path: P) -> BinResult<Self> {
        BufReader::new(std::fs::File::open(path)?).read_le()
    }
    pub fn find(&self, value: &str) -> LSEntry{
        let crchash = ls_str::crc32(value.as_bytes());
        let entry = self.ls_entry.iter().rposition(|n| n.crc == crchash);
        let mut return_value: LSEntry = LSEntry{
            crc : 0,
            offset : 0,
            size : 0,
            dt_index : None,
            padding : None
        };
        match entry{
            Some(x) => return_value = self.ls_entry[x],
            None => println!("Can not find:{}",value)
        }
        return_value
    }
}