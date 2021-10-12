
use std::path::PathBuf;
use std::path::Path;
use flate2::read::ZlibDecoder;
use std::io::BufReader;
use std::io::Seek;
use std::io::Read;
use std::io::SeekFrom;
use std::fs::File;
mod ls;

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
    //let d = File::open(_dt_file).unwrap();
    print!("RF Compressed is {0} Bytes\nRF Offset is {1}",rf_file_info.size,rf_file_info.offset);
    let  mut dt = BufReader::new(File::open(_dt_file).unwrap());
    dt.seek(SeekFrom::Start(rf_file_info.offset as u64));
    let mut rf_encode = vec![0u8;rf_file_info.size as usize];
    dt.read_exact(&mut rf_encode).unwrap();
    let rf_decoded = ZlibDecoder::new(&rf_encode[..]);
    //rf_decoded.read_vectored(bufs: &mut [IoSliceMut<'_>])

}