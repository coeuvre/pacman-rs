#[macro_use]
extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
mod win32;

#[cfg(windows)]
pub use win32::*;

#[derive(Debug)]
pub enum WindowEvent {
    CloseRequested,
}