use std;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::ffi::CStr;
use std::marker::PhantomData;

use failure::{err_msg, Error};
use stb::image::*;

fn base() -> PathBuf {
    // TODO(coeuvre): Other platform?
    std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("Resources").join("assets")
}

pub enum Image {
    RGBA8(Inner<Rgba8>),
    A8(Inner<A8>),
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
        Inner::<Rgba8>::load_from_memory_and_flip(buf).and_then(|inner| Ok(Image::RGBA8(inner)))
    }

    pub fn width(&self) -> usize {
        match *self {
            Image::RGBA8(ref inner) => inner.width,
            Image::A8(ref inner) => inner.width,
        }
    }

    pub fn height(&self) -> usize {
        match *self {
            Image::RGBA8(ref inner) => inner.height,
            Image::A8(ref inner) => inner.height,
        }
    }

    pub fn date(&self) -> &[u8] {
        match *self {
            Image::RGBA8(ref inner) => inner.data(),
            Image::A8(ref inner) => inner.data(),
        }
    }
}

pub trait Pixel {
    const NUM_CHANNEL: usize;
    const NUM_BYTES_PER_CHANNEL: usize;
    const NUM_BYTES: usize = Self::NUM_BYTES_PER_CHANNEL * Self::NUM_CHANNEL;
}

pub struct A8 {
    pub a: u8,
}

impl Pixel for A8 {
    const NUM_CHANNEL: usize = 1;
    const NUM_BYTES_PER_CHANNEL: usize = 1;
}

pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel for Rgba8 {
    const NUM_CHANNEL: usize = 4;
    const NUM_BYTES_PER_CHANNEL: usize = 1;
}

pub struct Inner<P: Pixel> {
    data: *mut u8,
    width: usize,
    height: usize,
    _phantom: std::marker::PhantomData<P>,
}

impl Inner<Rgba8> {
    fn load_from_memory_and_flip(buf: &[u8]) -> Result<Self, Error> {
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

            Ok(Inner {
                data,
                width: width as usize,
                height: height as usize,
                _phantom: PhantomData,
            })
        }
    }
}

impl<P: Pixel> Inner<P> {
    pub fn data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, P::NUM_BYTES * self.width * self.height) }
    }
}

impl<P: Pixel> Drop for Inner<P> {
    fn drop(&mut self) {
        trace!("Drop image");
        unsafe { stbi_image_free(self.data as *mut u8) };
    }
}
