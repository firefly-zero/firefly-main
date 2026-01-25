use crate::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::Rate;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{
    delay::Delay,
    dma_tx_buffer,
    gpio::{Level, Output, OutputConfig},
    lcd_cam::{lcd::i8080::I8080, LcdCam},
    rng::Rng,
    spi::master::Spi,
    uart::Uart,
};
use esp_println::println;
use firefly_hal::{Device, DeviceImpl};
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};

pub fn run_v1(peripherals: Peripherals) -> Result<(), Error> {
    println!("initializing display...");
    let display = {
        // hardware reset
        let rst = peripherals.GPIO46;
        let mut rst = Output::new(rst, Level::Low, OutputConfig::default());
        rst.set_high();

        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let config = esp_hal::lcd_cam::lcd::i8080::Config::default();
        let bus = I8080::new(lcd_cam.lcd, peripherals.DMA_CH0, config)
            .unwrap()
            .with_cs(peripherals.GPIO1)
            .with_data0(peripherals.GPIO9)
            .with_data1(peripherals.GPIO10)
            .with_data2(peripherals.GPIO11)
            .with_data3(peripherals.GPIO12)
            .with_data4(peripherals.GPIO13)
            .with_data5(peripherals.GPIO14)
            .with_data6(peripherals.GPIO21)
            .with_data7(peripherals.GPIO45)
            .with_data8(peripherals.GPIO38)
            .with_data9(peripherals.GPIO39)
            .with_data10(peripherals.GPIO40)
            .with_data11(peripherals.GPIO41)
            .with_data12(peripherals.GPIO42)
            .with_data13(peripherals.GPIO44)
            .with_data14(peripherals.GPIO43)
            .with_data15(peripherals.GPIO2)
            .with_dc(peripherals.GPIO8)
            .with_wrx(peripherals.GPIO3);
        // 2 bytes per pixel, 240 pixels per line, 4 lines.
        let buf1 = dma_tx_buffer!(480 * 4).unwrap();
        let buf2 = dma_tx_buffer!(480 * 4).unwrap();
        let writer = Writer::new(bus, buf1, buf2);
        Display::new(writer).unwrap()
    };

    println!("initializing SPIs...");
    let sd_spi = {
        let sclk = peripherals.GPIO15;
        let miso = peripherals.GPIO7;
        let mosi = peripherals.GPIO16;
        let cs = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());
        let pwr = peripherals.GPIO47;
        Output::new(pwr, Level::High, OutputConfig::default());
        Delay::new().delay_millis(10);

        let spi_config = esp_hal::spi::master::Config::default().with_frequency(Rate::from_mhz(4));
        let spi = Spi::new(peripherals.SPI2, spi_config)
            .unwrap()
            .with_sck(sclk)
            .with_miso(miso)
            .with_mosi(mosi);
        ExclusiveDevice::new(spi, cs, Delay::new()).unwrap()
    };

    let io_uart = {
        let miso = peripherals.GPIO4;
        let mosi = peripherals.GPIO5;
        let uart_config = esp_hal::uart::Config::default().with_baudrate(921_600);
        Uart::new(peripherals.UART1, uart_config)
            .unwrap()
            .with_rx(miso)
            .with_tx(mosi)
    };

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    _ = usb_serial.write_byte_nb(0x00);

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing device...");
    let rng = Rng::new();
    let device = DeviceImpl::new(sd_spi, io_uart, usb_serial, rng)?;
    let mut config = RuntimeConfig {
        id: None,
        device,
        display,
        net_handler: NetHandler::None,
    };

    config.device.log_debug("firmware", "running...");
    loop {
        let mut runtime = Runtime::new(config)?;
        runtime.start()?;
        loop {
            let exit = runtime.update()?;
            // Exit requested. Finalize runtime and get ownership of the device back.
            if exit {
                config = runtime.finalize()?;
                break;
            }
        }
    }
}
