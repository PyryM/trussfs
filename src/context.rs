use crate::archive::Archive;
use slotmap::{HopSlotMap, Key};
use std::convert::From;
use std::ffi::CString;
use std::fs;

type StringList = Vec<CString>;

slotmap::new_key_type! {
  pub struct ArchiveKey;
}
slotmap::new_key_type! {
  pub struct StringListKey;
}

// Eh, couldn't figure out how to make this generic
impl From<u64> for ArchiveKey {
    fn from(item: u64) -> Self {
        Self::from(slotmap::KeyData::from_ffi(item))
    }
}
impl From<ArchiveKey> for u64 {
    fn from(item: ArchiveKey) -> Self {
        item.data().as_ffi()
    }
}

impl From<u64> for StringListKey {
    fn from(item: u64) -> Self {
        Self::from(slotmap::KeyData::from_ffi(item))
    }
}
impl From<StringListKey> for u64 {
    fn from(item: StringListKey) -> Self {
        item.data().as_ffi()
    }
}

pub struct Context {
    pub archives: HopSlotMap<ArchiveKey, Archive>,
    pub stringlists: HopSlotMap<StringListKey, StringList>,
}

fn entry_filename(entry: std::fs::DirEntry, files_only: bool) -> Option<CString> {
    let path = entry.path();
    let metadata = fs::metadata(&path).ok()?;
    if files_only && !metadata.is_file() {
        return None;
    };
    let s = path.to_string_lossy().to_string();
    CString::new(s).ok()
}

impl Context {
    pub fn new() -> Self {
        Context {
            archives: HopSlotMap::with_key(),
            stringlists: HopSlotMap::with_key(),
        }
    }

    pub fn listdir(&mut self, path: String, files_only: bool) -> Option<StringListKey> {
        let mut items: Vec<CString> = Vec::new();
        for entry in fs::read_dir(path).ok()? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            if let Some(cstr) = entry_filename(entry, files_only) {
                items.push(cstr);
            };
        }
        Some(self.stringlists.insert(items))
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
