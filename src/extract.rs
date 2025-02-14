use std::fs::File;
use std::fs::*;
use std::io::BufReader;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::ls;
use crate::rf;
use ls::LSEntry;

struct FileToWrite {
    path: PathBuf,
    data: Vec<u8>,
    is_compressed: bool,
}

fn decode_zlib(bytes: &[u8]) -> Result<Vec<u8>, zune_inflate::errors::InflateDecodeErrors> {
    zune_inflate::DeflateDecoder::new(bytes).decode_zlib()
}

pub fn extract(ls_file: PathBuf, dt_file: PathBuf, dt1_file: Option<PathBuf>, out_folder: PathBuf) {
    let ls = ls::LSFile::open(ls_file).unwrap();

    let rf_file_info = ls.find("resource");

    let mut dts = Vec::new();
    dts.push(BufReader::new(File::open(&dt_file).unwrap()));
    if let Some(dt1_file) = &dt1_file {
        dts.push(BufReader::new(File::open(&dt1_file).unwrap()));
    }
    let mut dt_index = rf_file_info.dt_index.unwrap_or_default();
    dts[dt_index as usize]
        .seek(SeekFrom::Start(rf_file_info.offset as u64))
        .unwrap();

    // Alocate memory for rf
    let mut rf_data = vec![0u8; rf_file_info.size as usize];
    dts[dt_index as usize].read_exact(&mut rf_data).unwrap();
    let mut filetest = File::create("testOut.rf").unwrap();
    filetest.write_all(&rf_data).unwrap();

    // Read from buffer into that allocated memory
    let mut rf_cursor = Cursor::new(&rf_data);
    let rf = rf::RFFile::read(&mut rf_cursor);
    filetest.seek(SeekFrom::Start(0x80)).unwrap();
    filetest.write_all(&rf.debug_extract).unwrap();

    let out_paths = build_paths(&rf);

    let mut lsoffset: LSEntry = LSEntry::default();

    // Single producer for reading and multiple consumers for writing.
    // This ensures we only read from a single thread.
    let (sender, receiver) = crossbeam_channel::unbounded();

    let threads: Vec<_> = (0..num_cpus::get())
        .map(|_| {
            let receiver: crossbeam_channel::Receiver<FileToWrite> = receiver.clone();
            std::thread::spawn(move || {
                // Process files until the sender is dropped.
                while let Ok(file) = receiver.recv() {
                    let data = if file.is_compressed {
                        decode_zlib(&file.data).unwrap()
                    } else {
                        file.data
                    };

                    if let Err(e) = std::fs::write(&file.path, data) {
                        println!("Failed to extract {:?}: {e}", &file.path)
                    }
                }
            })
        })
        .collect();

    for (rf_entry, out_path) in rf.entrys.iter().zip(out_paths) {
        let path = out_folder.join(&out_path);

        // Create all necessary folders before extracting files.
        if rf_entry.is_folder && !path.exists() {
            create_dir_all(&path).unwrap();
        }

        if rf_entry.is_compressed {
            let mut cur_data = out_path.clone();
            if rf_entry.is_folder {
                cur_data.push(PathBuf::from("packed"));
            };
            lsoffset = ls.find(&format!(
                "data/{}",
                &cur_data.to_str().unwrap().replace('\\', "/")
            ));
            dt_index = lsoffset.dt_index.unwrap_or_default();
        } else if !rf_entry.is_folder {
            let mut data = vec![0u8; rf_entry.file_size_cmp as usize];
            dts[dt_index as usize]
                .seek(SeekFrom::Start(
                    (lsoffset.offset + rf_entry.file_offset) as u64,
                ))
                .unwrap();
            dts[dt_index as usize].read_exact(&mut data).unwrap();

            let is_compressed = rf_entry.file_size_cmp != rf_entry.file_size;

            let file_to_write = FileToWrite {
                path,
                data,
                is_compressed,
            };
            sender.send(file_to_write).unwrap();
        }
    }

    // Drop the sender to signal receiver threads to finish.
    drop(sender);

    for t in threads {
        t.join().unwrap();
    }
}

fn build_paths(rf: &rf::RFFile) -> Vec<PathBuf> {
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
    out_paths
}
