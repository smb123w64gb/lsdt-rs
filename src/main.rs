
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
use clap::Parser;
mod ls;
mod rf;

pub use ls::ls_str::crc32;

#[derive(Parser)]
struct Args {
    dt: PathBuf,
    ls: PathBuf,
    out_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("ls path: {:?}", args.ls);
    extract(args.ls,args.dt, args.out_dir);
}

fn extract(_ls_file: PathBuf, _dt_file: PathBuf,_out_folder: PathBuf) {
    let ls = ls::LSFile::open(_ls_file).unwrap(); 

    let rf_file_info = ls.find("resource");
    //Resorce file info get!

    //let d = File::open(_dt_file).unwrap();
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
    let mut stringbuild: Vec<PathBuf> = Vec::new();
    let mut path_out: Vec<PathBuf> = Vec::new();
    for n in &rf.entrys {
        while n.folder_depth  < stringbuild.len() as u32 {
            stringbuild.pop();
        }
        stringbuild.push(PathBuf::from(n.file_name.clone()));
     path_out.push(stringbuild.iter().collect());
    
    }
    let mut lsoffset: LSEntry= LSEntry::default();
    for n in 0..rf.entrys.len(){
        let mut cur_data = path_out[n].clone();
        let mut folder_path = _out_folder.join(&path_out[n]);
        if rf.entrys[n].is_folder{
            if !folder_path.exists() {
                create_dir(&folder_path);
            }
        }
        if rf.entrys[n].is_compressed{
            if rf.entrys[n].is_folder{
                folder_path.push(PathBuf::from("packed"));
                cur_data.push(PathBuf::from("packed"));
        };
            lsoffset = ls.find(&format!("data/{0}" ,&cur_data.to_str().unwrap()));
            let mut cur_data = vec![0u8;lsoffset.size as usize];
            dt.seek(SeekFrom::Start(lsoffset.offset as u64)).unwrap();
            dt.read_exact(&mut cur_data).unwrap();
            let mut fs_cursor = Cursor::new(&cur_data);
            println!("{0}",&folder_path.to_str().unwrap());
            std::fs::write(&folder_path,cur_data).unwrap();

    }
}

}