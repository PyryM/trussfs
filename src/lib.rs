use crate::context::Context;
use log::{error, info, warn};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

mod archive;
mod context;

const INVALID_HANDLE: u64 = u64::MAX;

fn c_str_to_string(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s).to_string_lossy().into_owned() }
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
pub unsafe extern "C" fn trussfs_list_dir(
    ctx: *mut Context,
    path: *const c_char,
    files_only: bool,
) -> u64 {
    let ctx = &mut *ctx;
    let path = c_str_to_string(path);
    match ctx.listdir(path, files_only) {
        Some(handle) => handle.into(),
        None => INVALID_HANDLE,
    }
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
