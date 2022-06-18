use std::path::PathBuf;
use std::*;
use std::io::BufReader;
use std::fs::File;
use std::fs::*;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::error::Error;

use flate2::read::ZlibDecoder;

use rayon::prelude::*;

use ls::LSEntry;
use crate::ls;
use crate::rf;

pub use ls::ls_str::crc32;

pub struct DecompressLater{
    pub path:PathBuf,
    pub data:Vec<u8>,
    pub cmp:bool,
}
fn decode_reader(bytes:&[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut z = ZlibDecoder::new(bytes);
    let mut s:Vec<u8> = Vec::new();
    z.read_to_end(&mut s)?;
    Ok(s)
}
pub fn extract(_ls_file: PathBuf, _dt_file: PathBuf, _dt1_file: PathBuf,_out_folder: PathBuf) {
    let ls = ls::LSFile::open(_ls_file).unwrap(); 

    let rf_file_info = ls.find("resource");

    let mut dts = Vec::new();
    dts.push(BufReader::new(File::open(_dt_file).unwrap()));
    if !_dt1_file.as_os_str().is_empty() {
        println!("{0}",_dt1_file.as_os_str().to_str().unwrap());
        dts.push(BufReader::new(File::open(_dt1_file).unwrap()));
    }
    let mut dtIdx;
    match rf_file_info.dt_index{
        Some(x)=> dtIdx=x,
        None => dtIdx=0,
    }
    dts[dtIdx as usize].seek(SeekFrom::Start(rf_file_info.offset as u64)).unwrap();

    let mut rf_data = vec![0u8;rf_file_info.size as usize];
    //Alocate memory for rf
    dts[dtIdx as usize].read_exact(&mut rf_data).unwrap();
    let mut filetest = File::create("testOut.rf").unwrap();
    filetest.write_all(&rf_data).unwrap();
    //Read from buffer into that alocated memory
    let mut rf_cursor = Cursor::new(&rf_data);
    let rf = rf::RFFile::read(&mut rf_cursor);
    filetest.seek(SeekFrom::Start(0x80)).unwrap();
    filetest.write_all(&rf.debug_extract).unwrap();
    let mut stringbuild: PathBuf = PathBuf::new();
    let mut path_out: Vec<PathBuf> = Vec::new();
    let mut oneshot = false;
    for n in &rf.entrys {
        while n.folder_depth < stringbuild.components().count() as u32+1 && oneshot{
            stringbuild.pop();
        }
        oneshot = true;
        stringbuild.push(PathBuf::from(&n.file_name));
        path_out.push(stringbuild.iter().collect());
    
    }
    let mut lsoffset: LSEntry= LSEntry::default();
    let mut dolater: Vec<DecompressLater> = Vec::new();
    for n in 0..rf.entrys.len(){
        
        let mut cur_data = path_out[n].clone();
        let mut folder_path = _out_folder.join(&path_out[n]);
        if rf.entrys[n].is_folder && !folder_path.exists() {create_dir_all(&folder_path);}
        if rf.entrys[n].is_compressed{
            if rf.entrys[n].is_folder{
                folder_path.push(PathBuf::from("packed"));
                cur_data.push(PathBuf::from("packed"));
        };
            lsoffset = ls.find(&format!("data/{0}" ,&cur_data.to_str().unwrap().replace('\\',"/")));
            match lsoffset.dt_index{
                Some(x)=> dtIdx=x,
                None => dtIdx=0,
            }
            let mut cur_data = vec![0u8;lsoffset.size as usize];
            dts[dtIdx as usize].seek(SeekFrom::Start(lsoffset.offset as u64)).unwrap();
            dts[dtIdx as usize].read_exact(&mut cur_data).unwrap();
            let _fs_cursor = Cursor::new(&cur_data);
            std::fs::write(&folder_path,cur_data).unwrap();

    }else if !rf.entrys[n].is_folder{
        let mut cur_cmp_data = vec![0u8;rf.entrys[n].file_size_cmp as usize];
        dts[dtIdx as usize].seek(SeekFrom::Start((lsoffset.offset + rf.entrys[n].file_offset) as u64)).unwrap();
        dts[dtIdx as usize].read_exact(&mut cur_cmp_data).unwrap();
        std::fs::write(&folder_path,if rf.entrys[n].file_size_cmp != rf.entrys[n].file_size{decode_reader(&cur_cmp_data[..]).unwrap()}else{cur_cmp_data});
    }
}
dolater.par_iter().for_each(|c|
    std::fs::write(&c.path,if c.cmp{
        decode_reader(&c.data[..]).unwrap() 
    }else{
        c.data.clone()
    } ).unwrap());

}