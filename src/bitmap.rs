pub trait Pixel {
    const CHANNELS: u32;
}

pub trait SRGBPixel : Pixel {
}

#[repr(C)]
pub struct RGBA8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel for RGBA8 {
    const CHANNELS: u32 = 4;
}

impl SRGBPixel for RGBA8 {
}

pub struct Bitmap<P> {
    width: u32,
    height: u32,
    /// num of pixels for a row
    stride: u32,
    pixels: Vec<P>,
}

impl<P: Pixel> Bitmap<P> {

}
