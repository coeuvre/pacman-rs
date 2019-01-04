use std::{
    fs
};

use failure::Error;

pub fn url_to_path(url: &str) -> String {
    const ASSET_PREFIX: &str = "assets://";
    const ASSET_DIR: &str = "assets/";
    const DATA_PREFIX: &str = "data://";
    const DATA_DIR: &str = "data/";

    let mut result = String::new();

    if url.starts_with(ASSET_PREFIX) {
        result.push_str(ASSET_DIR);
        result.push_str(&url[ASSET_PREFIX.len()..]);
    } else if url.starts_with(DATA_PREFIX) {
        result.push_str(DATA_DIR);
        result.push_str(&url[DATA_PREFIX.len()..]);
    } else {
        result.push_str(url);
    }

    result
}

pub fn read(url: &str) -> Result<Vec<u8>, Error> {
    Ok(fs::read(url_to_path(url))?)
}
