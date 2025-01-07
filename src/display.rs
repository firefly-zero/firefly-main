use core::convert::Infallible;

use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor::Rgb666,
    prelude::{DrawTarget, OriginDimensions},
};
use esp_hal::{lcd_cam::lcd::i8080::I8080, Blocking};

const WIDTH: usize = 480;
const HEIGHT: usize = 360;

pub struct Display<'a> {
    pub i8080: I8080<'a, Blocking>,
}

impl<'a> OriginDimensions for Display<'a> {
    fn size(&self) -> Size {
        Size {
            width: WIDTH as u32,
            height: HEIGHT as u32,
        }
    }
}

impl<'a> Display<'a> {
    fn set_pixel(&mut self, point: Point, color: Rgb666) {
        todo!()
    }

    pub fn flush(&mut self) {
        todo!()
    }
}

impl<'a> DrawTarget for Display<'a> {
    type Color = Rgb666;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let Pixel(point, color) = pixel;
            self.set_pixel(point, color);
        }
        Ok(())
    }
}
