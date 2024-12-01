use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

pub struct Display {}

impl Display {
    pub fn new() -> Self {
        Self {}
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size {
            width: 240,
            height: 180,
        }
    }
}

impl DrawTarget for Display {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        // ...
        Ok(())
    }
}
