extern crate gl;
#[macro_use]
extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;

#[cfg(windows)]
mod win_entry;
mod game;

#[cfg(windows)]
use win_entry as entry;

fn main() {
    entry::start();
}
