/// Software Reset.
///
/// When the Software Reset command is written, it causes software reset.
/// It resets commands and parameters to their S/W Reset default values.
/// (See default tables in each command description.) After the Software Reset
/// is applied, the display becomes blank immediately.
pub(super) const SWRESET: u8 = 0x01;

/// Sleep OUT.
///
/// This command turns off the sleep mode.
/// In this mode, the DC/DC converter is enabled, and
/// Internal oscillator and panel scanning are activated.
pub(super) const SLPOUT: u8 = 0x11;

/// Display ON.
///
/// This command causes the ILI9488 to start displaying the image data on the display device.
/// The frame memory contents remain unchanged. No status bits are changed.
pub(super) const DISON: u8 = 0x29;

/// Column Address Set.
///
/// This command is used to define the area of the frame memory that the MCU can access.
/// This command makes no change on the other driver status. The values of SC [15:0]
/// and EC [15:0] are referred when RAMWR command is applied. Each value represents
/// one column line in the Frame Memory.
pub(super) const CASET: u8 = 0x2A;

/// Page Address Set.
///
/// This command is used to define the area of the frame memory that the MCU can access.
/// This command makes no change on the other driver status. The values of SP [15:0]
/// and EP [15:0] are referred when RAMWR command is applied. Each value represents
/// one Page line in the Frame Memory.
pub(super) const PASET: u8 = 0x2B;

/// Memory Write.
///
/// This command transfers image data from the host processor to the ILI9488’s frame
/// memory starting at the pixel location specified by Column Address Set (2Ah)
/// and Page Address Set (2Bh) commands.
///
/// If Memory Access Control (36h) D5 = 0: The column and page registers are reset
/// to the Start Column (SC) and Start Page (SP), respectively. Pixel Data 1 is stored
/// in the frame memory at (SC, SP). The column register is then incremented and pixels
/// are written to the frame memory until the column register equals the End Column (EC)
/// value. The column register is then reset to SC and the page register is incremented.
/// Pixels are written to the frame memory until the page register equals the End Page
/// (EP) value or the host processor sends another command. If the number of pixels
/// exceeds (EC – SC + 1) * (EP – SP + 1), the extra pixels are ignored.
pub(super) const RAMWR: u8 = 0x2C;

/// Memory Access Control.
///
/// This command defines read/write scanning direction of the frame memory.
/// This command makes no change on other driver status.
pub(super) const MADCTL: u8 = 0x36;

/// Interface Pixel Format.
///
/// This command sets the pixel format for the RGB image data used by the interface.
/// DPI [2:0] is the pixel format selector of the RGB interface, and DBI [2:0]
/// is the pixel format of the MCU interface. If a particular interface, either
/// RGB interface or MCU interface, is not used then the corresponding bits
/// in the parameter are ignored.
pub(super) const COLMOD: u8 = 0x3A;

/// Memory Write Continue.
///
/// This command is used to transfer data from the MCU to the frame memory if the
/// frame memory wants to continue memory write after the “Memory Write (2Ch)” command.
/// This command makes no change to the other driver status. When this command
/// is accepted, the column register and the page register will not reset to the
/// Start Column/Start Page positions as it has been done by the “Memory Write (2Ch)”
/// command. Then D [23:0] is stored in the frame memory and the column register
/// and the page register incremented.
pub(super) const RAMWRC: u8 = 0x3C;
