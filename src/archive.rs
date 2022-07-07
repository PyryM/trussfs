use crate::context::StringList;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufReader, Read};
use zip::read::ZipFile;
use zip::ZipArchive;

pub struct Archive {
    zip: ZipArchive<BufReader<File>>,
}

fn format_zip_file_entry(idx: usize, file: &ZipFile) -> CString {
    let outpath = match file.enclosed_name() {
        Some(path) => path,
        None => return CString::new(format!("{} 0 X:", idx)).unwrap(),
    };

    let kind = if file.is_file() {
        'F'
    } else if file.is_dir() {
        'D'
    } else {
        '?'
    };
    let filesize = file.size();

    let s = format!(
        "{} {} {}:{}",
        idx,
        filesize,
        kind,
        outpath.to_string_lossy()
    );
    CString::new(s).unwrap()
}

fn read_zip_file(file: &mut ZipFile) -> Result<Vec<u8>, String> {
    let mut dest: Vec<u8> = Vec::with_capacity(file.compressed_size() as usize);
    match file.read_to_end(&mut dest) {
        Ok(_) => Ok(dest),
        Err(e) => Err(e.to_string()),
    }
}

impl Archive {
    pub fn open(filename: String) -> Result<Self, String> {
        let file = File::open(filename).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        Ok(Archive {
            zip: ZipArchive::new(reader).map_err(|e| e.to_string())?,
        })
    }

    pub fn list_files(&mut self) -> StringList {
        let mut filelist: StringList = Vec::new();
        for i in 0..self.zip.len() {
            if let Ok(file) = self.zip.by_index(i) {
                filelist.push(format_zip_file_entry(i, &file));
            };
        }
        filelist
    }

    pub fn filesize_by_index(&mut self, index: usize) -> Result<u64, String> {
        let file = self.zip.by_index(index).map_err(|e| e.to_string())?;
        Ok(file.size())
    }

    pub fn filesize_by_name(&mut self, filename: String) -> Result<u64, String> {
        let file = self.zip.by_name(&filename).map_err(|e| e.to_string())?;
        Ok(file.size())
    }

    pub fn read_file_by_index(&mut self, index: usize) -> Result<Vec<u8>, String> {
        let mut file = self.zip.by_index(index).map_err(|e| e.to_string())?;
        read_zip_file(&mut file)
    }

    pub fn read_file_by_name(&mut self, filename: String) -> Result<Vec<u8>, String> {
        let mut file = self.zip.by_name(&filename).map_err(|e| e.to_string())?;
        read_zip_file(&mut file)
    }
}
