use super::commands::*;
use super::writer::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::{prelude::*, primitives::Rectangle};
use esp_hal::dma::DmaTxBuf;

/// Pixels in a row (OX).
const REAL_WIDTH: u16 = 480;
/// Pixels in a column (OY).
const REAL_HEIGHT: u16 = 320;
const COLOR_BYTES: usize = 2;
const SCALE_Y: u8 = 2;
const SCALE_X: u8 = 2;

pub struct Display<'a> {
    writer: Writer<'a>,
}

impl OriginDimensions for Display<'_> {
    fn size(&self) -> Size {
        Size {
            width: u32::from(Self::WIDTH),
            height: u32::from(Self::HEIGHT),
        }
    }
}

impl<'a> Display<'a> {
    /// Pixels in a row (OX).
    pub const WIDTH: u16 = REAL_WIDTH / 2;
    /// Pixels in a column (OY).
    pub const HEIGHT: u16 = REAL_HEIGHT / 2;

    pub fn new(writer: Writer<'a>) -> Result<Self, Error> {
        let mut writer = writer;
        writer.send_cmd(SWRESET, [])?;
        writer.send_cmd(SLPOUT, [])?;
        writer.send_cmd(COLMOD, [0b101_0101])?;
        writer.send_cmd(MADCTL, [0b1110_0000])?;
        writer.send_cmd(DISON, [])?;
        Ok(Self { writer })
    }

    fn set_bounds(&mut self, area: &Rectangle) -> Result<Size, Error> {
        let tl = area.top_left;
        let w = area.size.width as i32;
        let h = area.size.height as i32;
        let bw = self.set_x_bounds(tl.x, tl.x + w - 1)?;
        let bh = self.set_y_bounds(tl.y, tl.y + h - 1)?;
        Ok(Size {
            width: u32::from(bw),
            height: u32::from(bh),
        })
    }

    fn set_x_bounds(&mut self, sc: i32, ec: i32) -> Result<u16, Error> {
        let max = i32::from(REAL_WIDTH) - 1;
        let sc = sc.clamp(0, max) as u16;
        let ec = ec.clamp(i32::from(sc), max) as u16;
        let params = [(sc >> 8) as _, sc as _, (ec >> 8) as _, ec as _];
        self.writer.send_cmd(CASET, params)?;
        Ok(ec - sc + 1)
    }

    fn set_y_bounds(&mut self, sp: i32, ep: i32) -> Result<u16, Error> {
        let max = i32::from(REAL_HEIGHT) - 1;
        let sp = sp.clamp(0, max) as u16;
        let ep = ep.clamp(i32::from(sp), max) as u16;
        let params = [(sp >> 8) as _, sp as _, (ep >> 8) as _, ep as _];
        self.writer.send_cmd(PASET, params)?;
        Ok(ep - sp + 1)
    }

    fn fill_buffer(&mut self, color: Rgb565) -> Result<DmaTxBuf, Error> {
        let raw = dump_color(color);
        let mut buf = self.writer.take_buffer()?;
        let bytes = buf.as_mut_slice();
        let bytes_len = bytes.len();
        for i in (0..bytes_len).step_by(2) {
            bytes[i..i + 2].copy_from_slice(&raw);
        }
        buf.set_length(bytes_len);
        Ok(buf)
    }
}

impl DrawTarget for Display<'_> {
    type Color = Rgb565;
    type Error = Error;

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        todo!()
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let area = Rectangle {
            top_left: Point {
                x: area.top_left.x * i32::from(SCALE_X),
                y: area.top_left.y * i32::from(SCALE_Y),
            },
            size: Size {
                width: area.size.width * u32::from(SCALE_X),
                height: area.size.height * u32::from(SCALE_Y),
            },
        };
        let size = self.set_bounds(&area)?;
        let n_pixels = (size.width * size.height / u32::from(SCALE_Y)) as usize;
        let mut buf = self.writer.take_buffer()?;
        let bytes_len = (size.width * 4) as usize;
        let mut bytes = &mut buf.as_mut_slice()[..bytes_len];
        let half_len = bytes_len / 2;
        let mut first = true;
        let mut cursor = 0;
        for color in colors.into_iter().take(n_pixels) {
            let raw = &dump_color(color);
            for _ in 0..SCALE_X {
                if cursor >= half_len {
                    let (left, right) = unsafe { bytes.split_at_mut_unchecked(half_len) };
                    right.copy_from_slice(left);
                    buf.set_length(bytes_len);
                    if first {
                        self.writer.send_data(RAMWR as u16, buf)?;
                        first = false;
                    } else {
                        self.writer.send_data(RAMWRC as u16, buf)?;
                    }
                    buf = self.writer.take_buffer()?;
                    bytes = &mut buf.as_mut_slice()[..bytes_len];
                    cursor = 0;
                }
                bytes[cursor] = raw[0];
                bytes[cursor + 1] = raw[1];
                cursor += COLOR_BYTES
            }
        }
        buf.set_length(cursor);
        // We must send buffer even if it is empty to return it to the pool.
        if first {
            self.writer.send_data(RAMWR as u16, buf)?;
        } else {
            self.writer.send_data(RAMWRC as u16, buf)?;
        }
        // for _ in 1..SCALE_Y {
        //     self.writer.send_data(RAMWRC as u16, buf)?;
        // }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = Rectangle {
            top_left: Point {
                x: area.top_left.x * i32::from(SCALE_X),
                y: area.top_left.y * i32::from(SCALE_Y),
            },
            size: Size {
                width: area.size.width * u32::from(SCALE_X),
                height: area.size.height * u32::from(SCALE_Y),
            },
        };
        let size = self.set_bounds(&area)?;
        self.writer.wait()?;
        let buf = self.fill_buffer(color)?;
        let bytes_len = buf.capacity();
        self.writer.send_data(RAMWR as u16, buf)?;
        let buf_pixels = (bytes_len / COLOR_BYTES) as u32;
        let n_pixels = size.width * size.height;
        for _ in 1..n_pixels / buf_pixels {
            // Wait for the previous write to finish to get reference to the same buffer.
            // Note that we don't wait for the last line.
            self.writer.wait()?;
            let buf = self.writer.take_buffer()?;
            self.writer.send_data(RAMWRC as u16, buf)?;
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.set_x_bounds(0, i32::from(REAL_WIDTH - 1))?;
        self.set_y_bounds(0, i32::from(REAL_HEIGHT - 1))?;
        self.writer.wait()?;
        let buf = self.fill_buffer(color)?;
        self.writer.send_data(RAMWR as u16, buf)?;
        for _ in 1..REAL_HEIGHT {
            // Wait for the previous write to finish to get reference to the same buffer.
            // Note that we don't wait for the last line.
            self.writer.wait()?;
            let buf = self.writer.take_buffer()?;
            self.writer.send_data(RAMWRC as u16, buf)?;
        }
        Ok(())
    }
}

fn dump_color(c: Rgb565) -> [u8; 2] {
    let r = u16::from(c.r());
    let g = u16::from(c.g());
    let b = u16::from(c.b());
    let raw = (b << 11) | (g << 5) | r;
    let raw = raw.to_le_bytes();
    [!raw[0], !raw[1]]
}
