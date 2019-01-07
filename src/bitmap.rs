use failure::{Error, format_err};

use stb_sys::image::*;

use std::{
    slice,
    ptr::*,
    ffi::CStr,
};
use crate::asset;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SRGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone)]
pub enum Pixels {
    SRGBA8(Vec<SRGBA8>),
    A8(Vec<u8>)
}

#[derive(Clone)]
pub struct Bitmap {
    pub width: i32,
    pub height: i32,
    /// num of pixels for a row
    pub stride: i32,
    pub pixels: Pixels,
}

impl Bitmap {
    pub fn from_url(url: &str) -> Result<Bitmap, Error> {
        let file_content = asset::read(url)?;
        let mut width = 0;
        let mut height = 0;
        let srgba8 = unsafe {
            let data = stbi_load_from_memory(
                file_content.as_ptr(), file_content.len() as i32,
                &mut width, &mut height,
                null_mut(), 4
            );
            if data.is_null() {
                return Err(format_err!("{}", CStr::from_ptr(stbi_failure_reason()).to_string_lossy()));
            }
            let srgba8 = slice::from_raw_parts(data, (width * height * 4) as usize).chunks(4).map(|chunk| {
                SRGBA8 {
                    r: *chunk.get_unchecked(0),
                    g: *chunk.get_unchecked(1),
                    b: *chunk.get_unchecked(2),
                    a: *chunk.get_unchecked(3),
                }
            }).collect::<Vec<_>>();

            stbi_image_free(data);

            srgba8
        };

        Ok(Bitmap {
            width,
            height,
            stride: width,
            pixels: Pixels::SRGBA8(srgba8),
        })
    }

    pub fn from_glyph(bitmap: &freetype::Bitmap) -> Bitmap {
        assert_eq!(bitmap.pixel_mode(), Ok(freetype::bitmap::PixelMode::Gray));
        assert_eq!(bitmap.width(), bitmap.pitch());

        Bitmap {
            width: bitmap.width(),
            height: bitmap.rows(),
            stride: bitmap.width(),
            pixels: Pixels::A8(Vec::from(bitmap.buffer())),
        }
    }
}
