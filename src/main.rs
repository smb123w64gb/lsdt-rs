
use std::*;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::fs::*;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::io::prelude::*;
use flate2::read::ZlibDecoder;
use ls::LSEntry;
mod ls;
mod rf;

pub use ls::ls_str::crc32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dt: PathBuf = args[1].as_str().into();
    let ls: PathBuf = args[2].as_str().into();
    let out_dir: PathBuf = args[3].as_str().into();
    extract(ls,dt, out_dir);
}

fn extract(_ls_file: PathBuf, _dt_file: PathBuf,_out_folder: PathBuf) {
    let ls = ls::LSFile::open(_ls_file).unwrap(); 

    let rf_file_info = ls.find("resource");
    //Resorce file info get!

    //let d = File::open(_dt_file).unwrap();
    //print!("RF Compressed is {0} Bytes\nRF Offset is {1}\n",rf_file_info.size,rf_file_info.offset);
    let mut dt = BufReader::new(File::open(_dt_file).unwrap());
    //Load our Data file into memory
    dt.seek(SeekFrom::Start(rf_file_info.offset as u64)).unwrap();
    //Seek to our RF file
    let mut rf_data = vec![0u8;rf_file_info.size as usize];
    //Alocate memory for rf
    dt.read_exact(&mut rf_data).unwrap();
    let mut filetest = File::create("testOut.rf").unwrap();
    filetest.write_all(&rf_data).unwrap();
    //Read from buffer into that alocated memory
    let mut rf_cursor = Cursor::new(&rf_data);
    let rf = rf::RFFile::read(&mut rf_cursor);
    filetest.seek(SeekFrom::Start(0x80)).unwrap();
    filetest.write_all(&rf.debug_extract).unwrap();
    let mut stringbuild: Vec<String> = Vec::new();
    let mut string_outs: Vec<String> = Vec::new();
    for n in &rf.entrys {
        while n.folder_depth  < stringbuild.len() as u32 {
            stringbuild.pop();
        }
        stringbuild.push(n.file_name.clone());
        string_outs.push(stringbuild.join(""))
    
    }
    let mut lsoffset: LSEntry= LSEntry{
        crc : 0,
        offset : 0,
        size : 0,
        dt_index : None,
        padding : None
    };
    for n in 0..rf.entrys.len(){
        if rf.entrys[n].is_folder{
            if Path::new(&format!("{0}\\{1}",_out_folder.to_str().unwrap(),string_outs[n])).exists() == false
            {
            create_dir(format!("{0}\\{1}",_out_folder.to_str().unwrap(),string_outs[n]));
            }
        }
        if rf.entrys[n].is_compressed{
            lsoffset = ls.find(&format!("data/{0}{1}",string_outs[n],if rf.entrys[n].is_compressed && rf.entrys[n].is_folder{"packed"}else{""}));
            let mut cur_data = vec![0u8;lsoffset.size as usize];
            dt.seek(SeekFrom::Start(lsoffset.offset as u64)).unwrap();
            dt.read_exact(&mut cur_data).unwrap();
            let mut fs_cursor = Cursor::new(&cur_data);
            //let mut fs_decomp = Vec::new();
            //let mut decomp_zlib = ZlibDecoder::new(fs_cursor);
            //decomp_zlib.read_to_end(&mut fs_decomp).unwrap();
            println!("{0}\\{1}",_out_folder.to_str().unwrap(),format!("{0}{1}",string_outs[n],if rf.entrys[n].is_compressed && rf.entrys[n].is_folder{"packed"}else{""}).replace("/","\\"));
            std::fs::write(format!("{0}\\{1}",_out_folder.to_str().unwrap(),format!("{0}{1}",string_outs[n],if rf.entrys[n].is_compressed && rf.entrys[n].is_folder{"packed"}else{""}).replace("/","\\")),cur_data).unwrap();

    }
}

}