use std;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::ffi::CStr;
use std::os::raw::*;

use failure::{err_msg, Error};
use stb::image::*;

fn base() -> PathBuf {
    // TODO(coeuvre): Other platform?
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("Resources")
        .join("assets")
}

pub enum ImageData {
    RGBA8(Vec<Rgba8>),
    A8(Vec<A8>),
}

impl ImageData {
    pub fn as_rgba8_ptr(&self) -> Result<*const u8, Error> {
        match *self {
            ImageData::RGBA8(ref data) => Ok(data.as_ptr() as *const u8),
            _ => Err(err_msg("Invalid data format")),
        }
    }
}

pub struct Image {
    data: ImageData,
    width: usize,
    height: usize,
}

impl Image {
    pub fn load_and_flip<P: AsRef<Path>>(path: P) -> Result<Image, Error> {
        let path = base().join(path);
        trace!("Loading image {}", path.display());

        let mut file = File::open(&path)?;

        let file_size = file.seek(io::SeekFrom::End(0))? as usize;
        file.seek(io::SeekFrom::Start(0))?;

        let mut buf = Vec::with_capacity(file_size);
        file.read_to_end(&mut buf)?;

        let image = Self::load_from_memory_and_flip(&buf)?;

        Ok(image)
    }

    pub fn load_from_memory_and_flip(buf: &[u8]) -> Result<Image, Error> {
        let i = StbImage::load_from_memory_and_flip(buf)?;
        assert!(i.num_channel == 4);

        let data = unsafe {
            std::slice::from_raw_parts(i.data as *const u32, (i.width * i.height) as usize).to_vec()
        };

        Ok(Image {
            data: ImageData::RGBA8(data),
            width: i.width as usize,
            height: i.height as usize,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &ImageData {
        &self.data
    }
}

pub trait Pixel {
    const NUM_CHANNEL: usize;
    const NUM_BYTES_PER_CHANNEL: usize;
    const NUM_BYTES: usize = Self::NUM_BYTES_PER_CHANNEL * Self::NUM_CHANNEL;
}

pub type A8 = u8;

impl Pixel for A8 {
    const NUM_CHANNEL: usize = 1;
    const NUM_BYTES_PER_CHANNEL: usize = 1;
}

pub type Rgba8 = u32;

impl Pixel for Rgba8 {
    const NUM_CHANNEL: usize = 4;
    const NUM_BYTES_PER_CHANNEL: usize = 1;
}

pub struct StbImage {
    data: *mut u8,
    width: c_int,
    height: c_int,
    num_channel: c_int,
}

impl StbImage {
    fn load_from_memory_and_flip(buf: &[u8]) -> Result<StbImage, Error> {
        let mut width = 0;
        let mut height = 0;
        let mut num_channel = 0;
        unsafe {
            stbi_set_flip_vertically_on_load(1);

            let data = stbi_load_from_memory(
                buf.as_ptr() as *mut u8,
                buf.len() as i32,
                &mut width,
                &mut height,
                &mut num_channel,
                Rgba8::NUM_CHANNEL as i32,
            );

            if data.is_null() {
                return Err(err_msg(
                    CStr::from_ptr(stbi_failure_reason())
                        .to_string_lossy()
                        .into_owned(),
                ));
            }

            Ok(StbImage {
                data,
                width,
                height,
                num_channel: Rgba8::NUM_CHANNEL as i32,
            })
        }
    }
}

impl Drop for StbImage {
    fn drop(&mut self) {
        unsafe { stbi_image_free(self.data as *mut u8) };
    }
}
