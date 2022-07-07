use crate::archive::Archive;
use slotmap::{HopSlotMap, Key};
use std::convert::From;
use std::ffi::CString;
use std::fs;

pub type StringList = Vec<CString>;

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
    pub last_error: Option<CString>,
    pub archives: HopSlotMap<ArchiveKey, Archive>,
    pub stringlists: HopSlotMap<StringListKey, StringList>,
}

fn format_entry(
    entry: std::fs::DirEntry,
    files_only: bool,
    include_metadata: bool,
) -> Option<CString> {
    let path = entry.path();
    let metadata = fs::metadata(&path).ok()?;
    if files_only && !metadata.is_file() {
        return None;
    };
    let s = path.to_string_lossy().to_string();
    let s = if include_metadata {
        // TODO: consider more metadata?
        // E.g., whether it's a symlink, modified time
        let prefix = if metadata.is_file() {
            'F'
        } else if metadata.is_dir() {
            'D'
        } else {
            '?'
        };
        let symlink = if metadata.is_symlink() { 'S' } else { '_' };
        format!("{} {}:{}", prefix, symlink, s)
    } else {
        s
    };
    CString::new(s).ok()
}

impl Context {
    pub fn new() -> Self {
        Context {
            last_error: None,
            archives: HopSlotMap::with_key(),
            stringlists: HopSlotMap::with_key(),
        }
    }

    pub fn mount_archive_err(&mut self, path: String) -> Result<ArchiveKey, String> {
        let archive = Archive::open(path)?;
        Ok(self.archives.insert(archive))
    }

    pub fn mount_archive(&mut self, path: String) -> Option<ArchiveKey> {
        match self.mount_archive_err(path) {
            Ok(archive) => Some(archive),
            Err(s) => {
                self.last_error = Some(CString::new(s).unwrap());
                None
            }
        }
    }

    pub fn list_archive(&mut self, archive: ArchiveKey) -> Option<StringListKey> {
        let archive = self.archives.get_mut(archive)?;
        Some(self.stringlists.insert(archive.list_files()))
    }

    pub fn listdir_err(
        &mut self,
        path: String,
        files_only: bool,
        include_metadata: bool,
    ) -> Result<StringListKey, String> {
        let mut items: Vec<CString> = Vec::new();
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            if let Some(cstr) = format_entry(entry, files_only, include_metadata) {
                items.push(cstr);
            };
        }
        Ok(self.stringlists.insert(items))
    }

    pub fn listdir(
        &mut self,
        path: String,
        files_only: bool,
        include_metadata: bool,
    ) -> Option<StringListKey> {
        match self.listdir_err(path, files_only, include_metadata) {
            Ok(strlist) => Some(strlist),
            Err(s) => {
                self.last_error = Some(CString::new(s).unwrap());
                None
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}