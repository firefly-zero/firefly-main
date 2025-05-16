use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb888},
    prelude::*,
};

#[derive(PartialEq, Clone, Copy)]
pub struct SmallRgb(u8, u8);

impl SmallRgb {
    pub fn from_rgb(r: u16, g: u16, b: u16) -> Self {
        let raw = (b << 11) | (g << 5) | r;
        let raw = raw.to_le_bytes();
        Self(!raw[0], !raw[1])
    }
}

impl PixelColor for SmallRgb {
    type Raw = RawU16;
}

impl From<Rgb888> for SmallRgb {
    fn from(c: Rgb888) -> Self {
        let r = u16::from(c.r());
        let g = u16::from(c.g());
        let b = u16::from(c.b());
        Self::from_rgb(r, g, b)
    }
}
