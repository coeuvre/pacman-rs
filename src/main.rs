#[cfg(windows)]
mod win_entry;

#[cfg(windows)]
use win_entry as entry;

fn main() {
    entry::start();
}
