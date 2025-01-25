/// No Operation.
///
/// This command is an empty command. It does not have any effect on the ILI9488.
/// However, it can be used to terminate Frame Memory Write or Read,
/// as described in RAMWR (Memory Write) and RAMRD (Memory Read) Commands.
pub(crate) const NOP: u8 = 0x00;

/// Software Reset.
///
/// When the Software Reset command is written, it causes software reset.
/// It resets commands and parameters to their S/W Reset default values.
/// (See default tables in each command description.) After the Software Reset
/// is applied, the display becomes blank immediately.
pub(crate) const SWRESET: u8 = 0x01;

/// Read display identification information.
///
/// This read byte can read 24 bits of display identification information.
///
/// * The 1st parameter is a dummy data.
/// * The 2nd parameter (ID1 [7:0]): LCD module’s manufacturer ID
/// * The 3rd parameter (ID2 [7:0]): LCD module/driver version ID
/// * The 4th parameter (ID3 [7:0]): LCD module/driver ID
pub(crate) const RDDIDIF: u8 = 0x04;

/// Read Number of the Errors on DSI.
///
/// * The 1st parameter is a dummy data.
/// * The 2nd parameter indicates the number of errors on the DSI.
///   More detailed description of the bits is below.
/// * P [6...0] bits indicate the number of the errors.
/// * P [7] is set to 1 if there is overflow with P [6..0] bits.
/// * P [7...0] bits are set to 0 (RDDSM(0Eh)’s D0 is set to 0 at the same time)
///   after the second parameter information is sent (=The read function is completed).
///   This function always returns P [7...0] = 00h if the parallel MCU interface is selected.
pub(crate) const RDNUMED: u8 = 0x05;

/// Read Display Status.
pub(crate) const RDDST: u8 = 0x09;

/// Read Display Power Mode.
pub(crate) const RDDPM: u8 = 0x0A;

/// Read Display MADCTL.
pub(crate) const RDDMADCTL: u8 = 0x0B;

/// Read Display Pixel Format (Read Display COLMOD).
pub(crate) const RDDCOLMOD: u8 = 0x0C;

/// Read Display Image Mode.
pub(crate) const RDDIM: u8 = 0x0D;

/// Read Display signal Mode.
pub(crate) const RDDSM: u8 = 0x0E;

/// Read Display Self-Diagnostic Result.
///
/// This command indicates the status of the display self-diagnostic
/// results after Sleep Out command.
pub(crate) const RDDSDR: u8 = 0x0F;

/// Sleep IN.
///
/// This command will cause the ILI9488 enter the minimum power consumption mode.
pub(crate) const SLPIN: u8 = 0x10;

/// Sleep OUT.
///
/// This command turns off the sleep mode.
/// In this mode, the DC/DC converter is enabled, and
/// Internal oscillator and panel scanning are activated.
pub(crate) const SLPOUT: u8 = 0x11;

/// Partial Mode ON.
///
/// This command will turn on the Partial mode.
/// The Partial mode window is described in the Partial Area command (30H).
/// To leave the Partial mode, the Normal Display Mode On command (13H) should be written.
pub(crate) const PTLON: u8 = 0x12;

/// Normal Display Mode ON.
///
/// This command will make the display return to the normal mode.
/// Normal Display Mode On means Partial mode off and Scroll mode off.
pub(crate) const NORON: u8 = 0x13;

/// Display Inversion OFF.
///
/// This command is used to recover from the Display Inversion mode.
/// Output from the Frame Memory is enabled.
/// This command makes no change to the content of the frame memory.
/// This command does not change any other status.
pub(crate) const INVOFF: u8 = 0x20;

/// Display Inversion ON.
///
/// This command is used to enter the Display Inversion mode.
/// This command makes no change of the content of the frame memory.
/// Every bit is inverted from the frame memory to the display.
/// This command does not change any other status.
/// To exit Display Inversion mode, the Display inversion OFF command (20h) should be written.
pub(crate) const INVON: u8 = 0x21;

/// All pixels OFF.
///
/// This command turns the display panel black in the Sleep Out mode,
/// and the status of the Display On/Off register can be On or Off.
/// This command makes no change to the contents of the Frame Memory.
/// This command does not change any other status.
///
/// To exit this mode, All Pixels On, Normal Display Mode On or Partial Mode On
/// commands can be used. The display panel shows the content of the Frame Memory
/// after applying Normal Display Mode On and Partial Mode On commands.
pub(crate) const ALLPOFF: u8 = 0x22;

/// All Pixels ON.
///
/// This command turns the display panel white in the Sleep Out mode,
/// and the status of the Display On/Off register can be On or Off.
/// This command makes no change to the contents of the Frame Memory.
/// This command does not change any other status.
///
/// To exit this mode, All Pixels Off, Normal Display Mode On or Partial Mode On
/// commands can be used. The display shows the content of the Frame Memory
/// after applying Normal Display Mode On and Partial Mode On commands.
pub(crate) const ALLPON: u8 = 0x23;

/// Display OFF.
///
/// This command causes the ILI9488 to stop displaying the image data on the display device.
/// The frame memory contents remain unchanged. No status bits are changed.
pub(crate) const DISOFF: u8 = 0x28;

/// Display ON.
///
/// This command causes the ILI9488 to start displaying the image data on the display device.
/// The frame memory contents remain unchanged. No status bits are changed.
pub(crate) const DISON: u8 = 0x29;

/// Column Address Set.
///
/// This command is used to define the area of the frame memory that the MCU can access.
/// This command makes no change on the other driver status. The values of SC [15:0]
/// and EC [15:0] are referred when RAMWR command is applied. Each value represents
/// one column line in the Frame Memory.
pub(crate) const CASET: u8 = 0x2A;

/// Page Address Set.
///
/// This command is used to define the area of the frame memory that the MCU can access.
/// This command makes no change on the other driver status. The values of SP [15:0]
/// and EP [15:0] are referred when RAMWR command is applied. Each value represents
/// one Page line in the Frame Memory.
pub(crate) const PASET: u8 = 0x2B;

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
pub(crate) const RAMWR: u8 = 0x2C;

/// Memory Read.
///
/// This command transfers image data from the ILI9488’s frame memory to the host
/// processor starting at the pixel location specified by set_column_address and
/// set_page_address commands.
///
/// If Memory Access control (36h) D5 = 0: The column and page registers are reset
/// to the Start Column (SC) and Start Page (SP), respectively. Pixels are read from
/// the frame memory at (SC, SP). The column register is then incremented and pixels
/// read from the frame memory until the column register equals the End Column (EC)
/// value. The column register is then reset to SC and the page register is incremented.
/// Pixels are read from the frame memory until the page register equals the End Page
/// (EP) value or the host processor sends another command.
pub(crate) const RAMRD: u8 = 0x2E;

/// Partial Area.
///
/// This command defines the Partial Display mode’s display area.
/// There are two parameters associated with this command:
/// the first defines the Start Row (SR) and the second the End Row (ER),
/// as illustrated in the figure in the datasheet. SR and ER refer to the Frame Memory.
pub(crate) const PLTAR: u8 = 0x30;

/// Vertical Scrolling Definition.
///
/// This command defines the display vertical scrolling area.
pub(crate) const VSCRDEF: u8 = 0x33;

/// Tearing Effect Line OFF.
///
/// This command turns off the ILI9488’s Tearing Effect output signal on the TE signal line.
pub(crate) const TEOFF: u8 = 0x34;

/// Tearing Effect Line ON.
///
/// This command is used to turn ON the Tearing Effect output signal from the TE signal line.
/// Changing the MADCTL bit D4 will not affect this output. The Tearing Effect Line On
/// has one parameter, which describes the mode of the Tearing Effect Output Line.
pub(crate) const TEON: u8 = 0x35;

/// Memory Access Control.
///
/// This command defines read/write scanning direction of the frame memory.
/// This command makes no change on other driver status.
pub(crate) const MADCTL: u8 = 0x36;

/// Vertical Scrolling Start Address.
///
/// This command is used together with Vertical Scrolling Definition (33h).
/// These two commands describe the scrolling area and the scrolling mode.
/// The Vertical Scrolling Start Address command has one parameter which
/// describes the address of the line in the Frame Memory that will be written
/// as the first line after the last line of the Top Fixed Area on the display.
pub(crate) const VSCRSADD: u8 = 0x37;

/// Idle Mode OFF.
///
/// This command causes the ILI9488 to exit the Idle mode. In the Idle Mode OFF,
/// the display panel can display a maximum of 262,144 colors.
pub(crate) const IDMOFF: u8 = 0x38;

/// Idle Mode ON.
///
/// This command is used to enter the Idle Mode On.
/// In the Idle Mode On, color expression is reduced.
/// The first bits of R, G, and B in the Frame Memory will determine the display color.
pub(crate) const IDMON: u8 = 0x39;

/// Interface Pixel Format.
///
/// This command sets the pixel format for the RGB image data used by the interface.
/// DPI [2:0] is the pixel format selector of the RGB interface, and DBI [2:0]
/// is the pixel format of the MCU interface. If a particular interface, either
/// RGB interface or MCU interface, is not used then the corresponding bits
/// in the parameter are ignored.
pub(crate) const COLMOD: u8 = 0x3A;

/// Memory Write Continue.
///
/// This command is used to transfer data from the MCU to the frame memory if the
/// frame memory wants to continue memory write after the “Memory Write (2Ch)” command.
/// This command makes no change to the other driver status. When this command
/// is accepted, the column register and the page register will not reset to the
/// Start Column/Start Page positions as it has been done by the “Memory Write (2Ch)”
/// command. Then D [23:0] is stored in the frame memory and the column register
/// and the page register incremented.
pub(crate) const RAMWRC: u8 = 0x3C;

/// Memory Read Continue.
///
/// This command is used to transfer data from the frame memory to the MCU, if the
/// MCU wants to continue memory read after “Memory Read (2Eh)” command. This command
/// makes no change to the other driver status. When this command is accepted, the
/// column register and the page register will not reset to the Start Column/Start Page
/// positions as it has been done by the “Memory Read (2Eh)” command. Then D [23:0]
/// is read back from the frame memory, and the column register and the page register
/// are incremented.
pub(crate) const RAMRDRC: u8 = 0x3e;

/// Write Tear Scan Line.
///
/// This command turns on the display Tearing Effect output signal on the TE signal
/// line when the display reaches line N. Changing Memory Access Control (36h) D4
/// will not affect the TE signal. The Tearing Effect Line On has one parameter
/// that describes the Tearing Effect Output Line mode. The Tearing Effect Output
/// line consists of V-Blanking information only.
///
/// Note that set Tear Scan Line with N = 0 is equivalent to Tearing Effect Line ON
/// with M = 0. The Tearing Effect Output line shall be active low when the ILI9488
/// is in the Sleep mode.
pub(crate) const TESLWR: u8 = 0x44;

/// Read Scan Line.
///
/// The display reads the current scan line N, used to update the display device.
/// The total number of scan lines on a display device is defined as:
/// V_Low + VBP + VACT + VFP. The first scan line is defined as the first
/// line of V-Sync and is denoted as Line 0.
///
/// When in the Sleep Mode, the value returned by Read Scan Line command is undefined.
pub(crate) const TESLRD: u8 = 0x45;

/// Write Display Brightness value.
///
/// This command is used to adjust the brightness value of the display.
/// DBV [7:0]: 8 bit, for display brightness of the manual brightness setting
/// and the CABC in the ILI9488. PWM output signal and PWM_OUT pin control the
/// LED driver IC in order to control the display brightness. In principle,
/// 00h value means the lowest brightness and FFh value means the highest brightness.
pub(crate) const WRDISBV: u8 = 0x51;

/// Read Display Brightness Value.
///
/// This command is used to return the brightness value of the display.
///
/// * DBV [7:0] is reset when the display is in the Sleep In mode.
/// * DBV [7:0] is 0 when the bit BCTRL of Write CTRL Display (53h) command is 0.
/// * DBV [7:0] is the manual set brightness specified by the Write CTRL Display (53h)
///   command when the BCTRL bit is 1.
///
/// When the bit BCTRL of Write CTRL Display (53h) command is 1 and C1/C0 bit
/// of Write Content Adaptive Brightness Control (55h) command are 0, DBV [7:0] output
/// is the brightness value specified by the Write Display Brightness (51h) command.
pub(crate) const RDDISBV: u8 = 0x52;

/// Write CTRL Display value.
///
/// This command is used to control the display brightness.
pub(crate) const WRCTRLD: u8 = 0x53;

/// Read CTRL Display Value.
///
/// This command is used to control the display brightness.
pub(crate) const RDCTRLD: u8 = 0x54;

/// Write Content Adaptive Brightness Control Value.
///
/// This command is used to set parameters of image content based on the adaptive
/// brightness control functionality. The first 4 different modes are for content
/// adaptive image functionality.
pub(crate) const WRCABC: u8 = 0x55;

/// Read Content Adaptive Brightness Control Value.
///
/// This command is used to read the settings of image content based on the adaptive
/// brightness control functionality. The first 4 different modes are for the content
/// adaptive image functionality.
pub(crate) const RDCABC: u8 = 0x56;

/// Write CABC Minimum Brightness.
///
/// This command is used to set the minimum brightness value of the display for the
/// CABC function. CMB [7:0]: CABC minimum brightness control, this parameter is used
/// to avoid too much brightness reduction. When the CABC is active, it cannot reduce
/// the display brightness to less than the CABC minimum brightness setting.
/// Image processing function works normally, even if the brightness cannot be changed.
/// This manual brightness setting does not affect other functions.
/// Manual brightness can set the display brightness to less than the CABC minimum
/// brightness. Smooth transition and dimming function can work normally.
/// When display brightness is turned off (BCTRL = 0 of Write CTRL Display (53h)),
/// the CABC minimum brightness setting is ignored.
///
/// In principle, 00h value means the lowest brightness for CABC,
/// and FFh value means the highest brightness for CABC.
pub(crate) const WRCABCMB: u8 = 0x5E;

/// Read CABC Minimum Brightness.
///
/// This command reads the minimum brightness value of the CABC function.
/// In principle, 00h value means the lowest brightness and FFh value means
/// the highest brightness. CMB [7:0] is the CABC minimum brightness specified
/// by the Write CABC minimum brightness (5Eh) command.
pub(crate) const RDCABCMB: u8 = 0x5F;

/// Read automatic brightness control self-diagnostic result.
///
/// This command indicates the status of the display self-diagnostic results for
/// automatic brightness control after the Sleep Out command.
pub(crate) const RDABCSDR: u8 = 0x68;

/// Read ID1.
///
/// This read byte identifies the LCD module’s manufacturer ID and it is specified by users.
///
/// The 1st parameter is a dummy data.
/// The 2nd parameter is the LCD module’s manufacturer ID.
pub(crate) const RDID1: u8 = 0xDA;

/// Read ID2.
///
/// This read byte is used to track the LCD module/driver version.
/// It is defined by the display supplier (with User’s agreement) and changes each time
/// a revision is made to the display, material or construction specifications.
///
/// The 1st parameter is a dummy data.
/// The 2nd parameter is the LCD module/driver version ID, and the ID parameter
/// range is from 80h to FFh.
///
/// The ID2 can be programmed by the OTP function.
pub(crate) const RDID2: u8 = 0xDB;

/// Read ID3.
///
/// This read byte identifies the LCD module/driver, and It is specified by users.
///
/// The 1st parameter is a dummy data.
/// The 2nd parameter is the LCD module/driver ID.
/// The ID3 can be programmed by the OTP function.
pub(crate) const RDID3: u8 = 0xDC;
