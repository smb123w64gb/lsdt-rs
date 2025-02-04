use std::fs::File;
use std::fs::*;
use std::io::BufReader;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::*;

use rayon::prelude::*;

use crate::ls;
use crate::rf;
use ls::LSEntry;

pub struct FileData {
    pub path: PathBuf,
    pub data: Vec<u8>,
    pub cmp: bool,
}

fn decode_zlib(bytes: &[u8]) -> Result<Vec<u8>, zune_inflate::errors::InflateDecodeErrors> {
    zune_inflate::DeflateDecoder::new(bytes).decode_zlib()
}

pub fn extract(ls_file: PathBuf, dt_file: PathBuf, dt1_file: PathBuf, out_folder: PathBuf) {
    let ls = ls::LSFile::open(ls_file).unwrap();

    let rf_file_info = ls.find("resource");

    let mut dts = Vec::new();
    dts.push(BufReader::new(File::open(dt_file).unwrap()));
    if !dt1_file.as_os_str().is_empty() {
        println!("{0}", dt1_file.as_os_str().to_str().unwrap());
        dts.push(BufReader::new(File::open(dt1_file).unwrap()));
    }
    let mut dt_idx;
    match rf_file_info.dt_index {
        Some(x) => dt_idx = x,
        None => dt_idx = 0,
    }
    dts[dt_idx as usize]
        .seek(SeekFrom::Start(rf_file_info.offset as u64))
        .unwrap();

    // Alocate memory for rf
    let mut rf_data = vec![0u8; rf_file_info.size as usize];
    dts[dt_idx as usize].read_exact(&mut rf_data).unwrap();
    let mut filetest = File::create("testOut.rf").unwrap();
    filetest.write_all(&rf_data).unwrap();

    // Read from buffer into that allocated memory
    let mut rf_cursor = Cursor::new(&rf_data);
    let rf = rf::RFFile::read(&mut rf_cursor);
    filetest.seek(SeekFrom::Start(0x80)).unwrap();
    filetest.write_all(&rf.debug_extract).unwrap();

    let mut stringbuild: PathBuf = PathBuf::new();
    let mut out_paths: Vec<PathBuf> = Vec::new();
    let mut oneshot = false;
    for n in &rf.entrys {
        while n.folder_depth < stringbuild.components().count() as u32 + 1 && oneshot {
            stringbuild.pop();
        }
        oneshot = true;
        stringbuild.push(PathBuf::from(&n.file_name));
        out_paths.push(stringbuild.iter().collect());
    }

    let mut lsoffset: LSEntry = LSEntry::default();
    let mut files_to_write: Vec<FileData> = Vec::new();

    // Create all necessary folders before extracting files.
    for (rf_entry, out_path) in rf.entrys.iter().zip(&out_paths) {
        let path = out_folder.join(out_path);
        if rf_entry.is_folder && !path.exists() {
            create_dir_all(&path).unwrap();
        }
    }

    for (rf_entry, out_path) in rf.entrys.iter().zip(out_paths) {
        let path = out_folder.join(&out_path);
        if rf_entry.is_compressed {
            let mut cur_data = out_path.clone();
            if rf_entry.is_folder {
                cur_data.push(PathBuf::from("packed"));
            };
            lsoffset = ls.find(&format!(
                "data/{0}",
                &cur_data.to_str().unwrap().replace('\\', "/")
            ));
            match lsoffset.dt_index {
                Some(x) => dt_idx = x,
                None => dt_idx = 0,
            }
        } else if !rf_entry.is_folder {
            let mut cur_cmp_data = vec![0u8; rf_entry.file_size_cmp as usize];
            dts[dt_idx as usize]
                .seek(SeekFrom::Start(
                    (lsoffset.offset + rf_entry.file_offset) as u64,
                ))
                .unwrap();
            dts[dt_idx as usize].read_exact(&mut cur_cmp_data).unwrap();

            files_to_write.push(FileData {
                path,
                data: cur_cmp_data,
                cmp: rf_entry.file_size_cmp != rf_entry.file_size,
            });
        }
    }

    files_to_write.par_iter().for_each(|c| {
        if let Err(e) = std::fs::write(
            &c.path,
            if c.cmp {
                decode_zlib(&c.data[..]).unwrap()
            } else {
                c.data.clone()
            },
        ) {
            println!("Failed to extract {:?}: {e}", &c.path)
        }
    });
}
