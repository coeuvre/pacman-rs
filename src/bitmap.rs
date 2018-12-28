pub trait Pixel {
    const CHANNELS: u32;
}

pub trait SRGBAPixel: Pixel {
    fn srgba(&self) -> Vec<u8>;
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

impl SRGBAPixel for RGBA8 {
    fn srgba(&self) -> Vec<u8> {
        vec![self.r, self.g, self.b, self.a]
    }
}

pub struct Bitmap<P> {
    pub width: u32,
    pub height: u32,
    /// num of pixels for a row
    pub stride: u32,
    pub pixels: Vec<P>,
}

impl<P: Pixel> Bitmap<P> {

}
