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

/// This command is an empty command. It does not have any effect on the ILI9488.
/// However, it can be used to terminate Frame Memory Write or Read,
/// as described in RAMWR (Memory Write) and RAMRD (Memory Read) Commands.
const NOP: u8 = 0x00;
/// When the Software Reset command is written, it causes software reset.
/// It resets commands and parameters to their S/W Reset default values.
/// (See default tables in each command description.) After the Software Reset
/// is applied, the display becomes blank immediately.
const SOFT_RESET: u8 = 0x01;
/// Read display identification information
///
/// This read byte can read 24 bits of display identification information.
///
/// * The 1st parameter is a dummy data.
/// * The 2nd parameter (ID1 [7:0]): LCD module’s manufacturer ID
/// * The 3rd parameter (ID2 [7:0]): LCD module/driver version ID
/// * The 4th parameter (ID3 [7:0]): LCD module/driver ID
const READ_ID: u8 = 0x04;
/// Read Number of the Errors on DSI;
const READ_ERRORS_NUMBER: u8 = 0x05;
/// Read Display Status
const READ_DISPLAY_STATUS: u8 = 0x09;
/// Read Display Power Mode
const READ_DISPLAY_POWER_MODE: u8 = 0x0A;
/// Read Display MADCTL
const READ_DISPLAY_MADCTL: u8 = 0x0B;
/// Read Pixel Format
const READ_PIXEL_FORMAT: u8 = 0x0C;
/// Read Display Image Mode
const READ_DISPLAY_IMAGE_MODE: u8 = 0x0D;
/// Read Display signal Mode
const READ_DISPLAY_SIGNAL_MODE: u8 = 0x0E;
/// Read Display Self-Diagnostic Result
const READ_SELF_DIAGNOSTIC: u8 = 0x0F;
// Sleep IN
const SLEEP_IN: u8 = 0x10;
// Sleep OUT
const SLEEP_OUT: u8 = 0x11;
/// Partial Mode ON
const PARTIAL_MODE_ON: u8 = 0x12;
// Normal Display Mode ON
const NORMAL_DISPLAY_MODE_ON: u8 = 0x13;
// Display Inversion OFF
const DISPLAY_INVERSION_OFF: u8 = 0x20;
const DISPLAY_INVERSION_ON: u8 = 0x21;
const ALL_PIXEL_OFF: u8 = 0x22;
const ALL_PIXEL_ON: u8 = 0x23;
const DISPLAY_OFF: u8 = 0x28;
const DISPLAY_ON: u8 = 0x29;
const COLUMN_ADDRESS_SET: u8 = 0x2A;
const PAGE_ADDRESS_SET: u8 = 0x2B;
const MEMORY_WRITE: u8 = 0x2C;
const MEMORY_READ: u8 = 0x2E;
const PARTIAL_AREA: u8 = 0x30;
/// Vertical Scrolling Definition
const VERTICAL_SCROLLING_DEFINITION: u8 = 0x33;
/// Tearing Effect Line OFF
const TEARING_EFFECT_LINE_OFF: u8 = 0x34;
/// Tearing Effect Line ON
const TEARING_EFFECT_LINE_ON: u8 = 0x35;
const MEMORY_ACCESS_CONTROL: u8 = 0x36;
const VERTICAL_SCROLLING_START_ADDRESS: u8 = 0x37;
const IDLE_MODE_OFF: u8 = 0x38;
const IDLE_MODE_ON: u8 = 0x39;
const INTERFACE_PIXEL_FORMAT: u8 = 0x3A;
const MEMORY_WRITE_CONTINUE: u8 = 0x3C;
const WRITE_TEAR_SCAN_LINE: u8 = 0x44;
const READ_TEAR_SCAN_LINE: u8 = 0x45;
/// Write Display Brightness value
const WRITE_DISPLAY_BRIGHTNESS_VALUE: u8 = 0x51;
const READ_DISPLAY_BRIGHTNESS_VALUE: u8 = 0x52;
/// Write CTRL Display value
const WRITE_CTRL_DISPLAY_VALUE: u8 = 0x53;
const READ_CTRL_DISPLAY_VALUE: u8 = 0x54;
/// Write Content Adaptive Brightness Control value
const WRITE_CONTENT_ADAPTIVE_BRIGHTNESS_CONTROL_VALUE: u8 = 0x55;
const READ_CONTENT_ADAPTIVE_BRIGHTNESS_CONTROL_VALUE: u8 = 0x56;
/// Write CABC Minimum Brightness
const WRITE_CABC_MINIMUM_BRIGHTNESS: u8 = 0x5E;
const READ_CABC_MINIMUM_BRIGHTNESS: u8 = 0x5F;
/// Read automatic brightness control self-diagnostic result
const READ_AUTOMATIC_BRIGHTNESS_CONTROL_SELF_DIAGNOSTIC_RESULT: u8 = 0x68;
/// Read ID1
const READ_ID1: u8 = 0xDA;
/// Read ID2
const READ_ID2: u8 = 0xDB;
/// Read ID3
const READ_ID3: u8 = 0xDC;
