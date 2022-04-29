
use std::*;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs::File;

use std::fs::*;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use flate2::read::ZlibDecoder;

use ls::LSEntry;
use clap::Parser;
mod ls;
mod rf;

pub use ls::ls_str::crc32;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    dt: PathBuf,
    ls: PathBuf,
    #[clap(short, long, default_value = "Out")]
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

    let mut dt = BufReader::new(File::open(_dt_file).unwrap());

    dt.seek(SeekFrom::Start(rf_file_info.offset as u64)).unwrap();

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
            let mut cur_data = vec![0u8;lsoffset.size as usize];
            dt.seek(SeekFrom::Start(lsoffset.offset as u64)).unwrap();
            dt.read_exact(&mut cur_data).unwrap();
            let _fs_cursor = Cursor::new(&cur_data);
            println!("{0}",&folder_path.to_str().unwrap());
            std::fs::write(&folder_path,cur_data).unwrap();

    }else if !rf.entrys[n].is_folder{
        let mut cur_cmp_data = vec![0u8;rf.entrys[n].file_size_cmp as usize];
        let mut cur_data= vec![0u8;rf.entrys[n].file_size as usize];
        dt.seek(SeekFrom::Start((lsoffset.offset + rf.entrys[n].file_offset) as u64)).unwrap();
        dt.read_exact(&mut cur_cmp_data).unwrap();
        if(rf.entrys[n].file_size_cmp != rf.entrys[n].file_size){
        let mut decomp_zlib = ZlibDecoder::new(&cur_cmp_data[..]);
        decomp_zlib.read_to_end(&mut cur_data).unwrap();
        std::fs::write(&folder_path,cur_data).unwrap();
        }else{
            std::fs::write(&folder_path,cur_cmp_data).unwrap();
        }
        println!("{0}",&folder_path.to_str().unwrap());

    }
}

}