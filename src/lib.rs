use crate::context::Context;
use log::{error, info, warn};
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::ptr;

mod archive;
mod context;
mod watcher;

const INVALID_HANDLE: u64 = u64::MAX;
const VERSION_NUMBER: u64 = 200; // 0.2.0

fn c_str_to_string(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
}

fn c_str_to_cstring(s: *const c_char) -> CString {
    CString::new(c_str_to_string(s)).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn trussfs_version() -> u64 {
    // It turns out having a constant null-terminated string in Rust
    // is more complicated than necessary so the version is a simple
    // number.
    VERSION_NUMBER
}

#[no_mangle]
pub extern "C" fn trussfs_is_handle_valid(handle: u64) -> bool {
    handle != INVALID_HANDLE
}

#[no_mangle]
pub extern "C" fn trussfs_init() -> *mut Context {
    info!("Creating new context.");
    println!("Creating new context.");
    if let Err(err) = env_logger::try_init() {
        warn!("Logger already initialized: {}", err)
    }
    Box::into_raw(Box::new(Context::new()))
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_shutdown(ctx: *mut Context) {
    info!("Requested ctx close!");
    let ctx = &mut *ctx;

    // take ownership and drop
    let b = Box::from_raw(ctx);
    drop(b);
    info!("Everything should be dead now!");
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_get_error(ctx: *mut Context) -> *const c_char {
    let ctx = &mut *ctx;
    ctx.last_error.as_ptr()
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_clear_error(ctx: *mut Context) {
    let ctx = &mut *ctx;
    ctx.clear_error();
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_recursive_makedir(_ctx: *mut Context, path: *const c_char) -> u64 {
    let path = c_str_to_string(path);
    match fs::create_dir_all(path) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_binary_dir(ctx: *mut Context) -> *const c_char {
    let ctx = &mut *ctx;
    ctx.update_dirs();
    match &ctx.binary_dir {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_working_dir(ctx: *mut Context) -> *const c_char {
    let ctx = &mut *ctx;
    ctx.update_dirs();
    match &ctx.working_dir {
        Some(s) => s.as_ptr(),
        None => ptr::null(),
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_watcher_create(
    ctx: *mut Context,
    path: *const c_char,
    recursive: bool,
) -> u64 {
    let ctx = &mut *ctx;
    let path = c_str_to_string(path);
    match ctx.watch_path(path, recursive) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_watcher_augment(
    ctx: *mut Context,
    watcher_handle: u64,
    path: *const c_char,
    recursive: bool,
) -> bool {
    let ctx = &mut *ctx;
    let path = c_str_to_string(path);
    ctx.watch_augment(watcher_handle.into(), path, recursive)
        .is_ok()
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_watcher_free(ctx: *mut Context, watcher_handle: u64) {
    let ctx = &mut *ctx;
    ctx.watchers.remove(watcher_handle.into());
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_watcher_poll(ctx: *mut Context, watcher: u64) -> u64 {
    let ctx = &mut *ctx;
    match ctx.watcher_poll(watcher.into()) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_mount(ctx: *mut Context, path: *const c_char) -> u64 {
    let ctx = &mut *ctx;
    let path = c_str_to_string(path);
    match ctx.mount_archive(path) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_list(ctx: *mut Context, archive: u64) -> u64 {
    let ctx = &mut *ctx;
    match ctx.list_archive(archive.into()) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_free(ctx: *mut Context, archive_handle: u64) {
    let ctx = &mut *ctx;
    ctx.archives.remove(archive_handle.into());
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_filesize_name(
    ctx: *mut Context,
    archive_handle: u64,
    name: *const c_char,
) -> u64 {
    let ctx = &mut *ctx;
    let name = c_str_to_string(name);
    match ctx.archives.get_mut(archive_handle.into()) {
        Some(archive) => archive.filesize_by_name(name).unwrap_or_default(),
        None => 0,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_filesize_index(
    ctx: *mut Context,
    archive_handle: u64,
    index: u64,
) -> u64 {
    let ctx = &mut *ctx;
    match ctx.archives.get_mut(archive_handle.into()) {
        Some(archive) => archive
            .filesize_by_index(index as usize)
            .unwrap_or_default(),
        None => 0,
    }
}

unsafe fn copy_data(data: Vec<u8>, dest: *mut u8, dest_size: u64) -> i64 {
    let ncopy = data.len();
    if ncopy > dest_size as usize {
        return -1;
    }
    ptr::copy_nonoverlapping(data.as_ptr(), dest, ncopy);
    ncopy as i64
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_read_name(
    ctx: *mut Context,
    archive_handle: u64,
    name: *const c_char,
    dest: *mut u8,
    dest_size: u64,
) -> i64 {
    let ctx = &mut *ctx;
    let archive = match ctx.archives.get_mut(archive_handle.into()) {
        Some(archive) => archive,
        None => return -1,
    };
    match archive.read_file_by_name(c_str_to_string(name)) {
        Ok(data) => copy_data(data, dest, dest_size),
        Err(_) => -1,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_archive_read_index(
    ctx: *mut Context,
    archive_handle: u64,
    index: u64,
    dest: *mut u8,
    dest_size: u64,
) -> i64 {
    let ctx = &mut *ctx;
    let archive = match ctx.archives.get_mut(archive_handle.into()) {
        Some(archive) => archive,
        None => return -1,
    };
    match archive.read_file_by_index(index as usize) {
        Ok(data) => copy_data(data, dest, dest_size),
        Err(_) => -1,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_list_dir(
    ctx: *mut Context,
    path: *const c_char,
    files_only: bool,
    include_metadata: bool,
) -> u64 {
    let ctx = &mut *ctx;
    let path = c_str_to_string(path);
    match ctx.listdir(path, files_only, include_metadata) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_split_path(ctx: *mut Context, path: *const c_char) -> u64 {
    let ctx = &mut *ctx;
    let path = c_str_to_string(path);
    match ctx.splitpath(path) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_list_new(ctx: *mut Context) -> u64 {
    let ctx = &mut *ctx;
    ctx.stringlists.insert(Vec::new()).into()
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_list_free(ctx: *mut Context, list_handle: u64) {
    let ctx = &mut *ctx;
    ctx.stringlists.remove(list_handle.into());
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_list_length(ctx: *mut Context, list_handle: u64) -> u64 {
    let ctx = &mut *ctx;
    if list_handle == INVALID_HANDLE {
        error!("Invalid list handle");
        return 0;
    };
    let strlist = match ctx.stringlists.get(list_handle.into()) {
        None => {
            warn!("List {} does not exist.", list_handle);
            return 0;
        }
        Some(list) => list,
    };
    strlist.len() as u64
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_list_get(
    ctx: *mut Context,
    list_handle: u64,
    list_index: u64,
) -> *const c_char {
    let ctx = &mut *ctx;
    if list_handle == INVALID_HANDLE {
        error!("Invalid list handle");
        return ptr::null();
    };
    let strlist = match ctx.stringlists.get(list_handle.into()) {
        None => {
            warn!("List {} does not exist.", list_handle);
            return ptr::null();
        }
        Some(list) => list,
    };
    let item = match strlist.get(list_index as usize) {
        None => {
            warn!(
                "Index {} out of range for list size {}",
                list_index,
                strlist.len()
            );
            return ptr::null();
        }
        Some(item) => item,
    };
    item.as_ptr()
}

/// # Safety
///
/// ctx must be valid
#[no_mangle]
pub unsafe extern "C" fn trussfs_list_push(
    ctx: *mut Context,
    list_handle: u64,
    item: *const c_char,
) -> u64 {
    let ctx = &mut *ctx;
    if list_handle == INVALID_HANDLE {
        error!("Invalid list handle");
        return 0;
    };
    match ctx.stringlists.get_mut(list_handle.into()) {
        None => {
            warn!("List {} does not exist.", list_handle);
            0
        }
        Some(list) => {
            list.push(c_str_to_cstring(item));
            1
        }
    }
}
