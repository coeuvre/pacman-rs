use std::ffi::{CStr, CString};
use std::fs;
use std::mem;
use std::ops::Deref;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr::null_mut;
use std::time::SystemTime;

use failure::{err_msg, Error};

pub struct DynamicLibrary {
    handle: *mut c_void,
    path: String,
    modified: SystemTime,
}

impl DynamicLibrary {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<DynamicLibrary, Error> {
        let path = path.as_ref().to_str().unwrap().to_string();
        let modified = modified(&path)?;
        let handle = load(&path)?;
        Ok(DynamicLibrary {
            handle,
            path,
            modified,
        })
    }

    pub fn is_modified(&self) -> bool {
        fs::metadata(&self.path)
            .and_then(|metadata| metadata.modified())
            .map(|modified| modified > self.modified)
            .unwrap_or(false)
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        unload(self.handle)?;
        self.modified = modified(&self.path)?;
        self.handle = load(&self.path)?;
        Ok(())
    }

    pub unsafe fn symbol<T>(&self, symbol: &str) -> Result<Symbol<T>, Error> {
        let symbol = CString::new(symbol).unwrap();
        let symbol = dlsym(self.handle, symbol.as_ptr());
        if symbol != null_mut() {
            Ok(Symbol {
                _dl: self,
                symbol: symbol as *mut T,
            })
        } else {
            Err(dl_err())
        }
    }
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        let _ = unload(self.handle);
    }
}

fn load(path: &str) -> Result<*mut c_void, Error> {
    let path = CString::new(path).unwrap();
    let handle = unsafe { dlopen(path.as_ptr(), RTLD_LAZY | RTLD_GLOBAL) };
    if handle != null_mut() {
        Ok(handle)
    } else {
        Err(dl_err())
    }
}

fn modified(path: &str) -> Result<SystemTime, Error> {
    let metadata = fs::metadata(&path)?;
    Ok(metadata.modified()?)
}

fn unload(handle: *mut c_void) -> Result<(), Error> {
    if unsafe { dlclose(handle) == 0 } {
        Ok(())
    } else {
        Err(dl_err())
    }
}

fn dl_err() -> Error {
    err_msg(unsafe { CStr::from_ptr(dlerror()).to_string_lossy().into_owned() })
}

pub struct Symbol<'dl, T: 'dl> {
    _dl: &'dl DynamicLibrary,
    pub symbol: *mut T,
}

impl<'dl, T> Deref for Symbol<'dl, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { mem::transmute(&self.symbol) }
    }
}

const RTLD_LAZY: c_int = 0x1;
const RTLD_GLOBAL: c_int = 0x8;

extern "C" {
    fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn dlerror() -> *const c_char;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
