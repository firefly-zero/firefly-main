use crate::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_storage::ReadStorage;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::AppPartitionSubType;
use esp_hal::peripherals::Peripherals;
use esp_hal::time::Rate;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{
    delay::Delay,
    dma_tx_buffer,
    gpio::{Level, Output, OutputConfig},
    lcd_cam::{lcd::i8080::I8080, LcdCam},
    psram::Psram,
    rng::Rng,
    spi::master::Spi,
    uart::Uart,
};
use esp_println::println;
use esp_storage::FlashStorage;
use firefly_hal::DeviceImpl;
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};
use firefly_types::DeviceInfo;

pub fn run_v2(peripherals: Peripherals) -> Result<(), Error> {
    let psram_config = esp_hal::psram::PsramConfig {
        mode: esp_hal::psram::PsramMode::OctalSpi,
        ..Default::default()
    };
    let psram = Psram::new(peripherals.PSRAM, psram_config);
    let (start, size) = psram.raw_parts();
    init_psram_heap(start, size);

    println!("initializing display...");
    let display = {
        // hardware reset
        let rst = peripherals.GPIO1;
        let mut rst = Output::new(rst, Level::Low, OutputConfig::default());
        rst.set_high();

        // Set display brightness to max.
        Output::new(peripherals.GPIO15, Level::High, OutputConfig::default());

        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let config = esp_hal::lcd_cam::lcd::i8080::Config::default();
        let bus = I8080::new(lcd_cam.lcd, peripherals.DMA_CH0, config)
            .unwrap()
            // .with_cs(peripherals.GPIO15)
            .with_data0(peripherals.GPIO42)
            .with_data1(peripherals.GPIO41)
            .with_data2(peripherals.GPIO40)
            .with_data3(peripherals.GPIO48)
            .with_data4(peripherals.GPIO47)
            .with_data5(peripherals.GPIO21)
            .with_data6(peripherals.GPIO14)
            .with_data7(peripherals.GPIO13)
            .with_data8(peripherals.GPIO12)
            .with_data9(peripherals.GPIO11)
            .with_data10(peripherals.GPIO10)
            .with_data11(peripherals.GPIO9)
            .with_data12(peripherals.GPIO46)
            .with_data13(peripherals.GPIO3)
            .with_data14(peripherals.GPIO8)
            .with_data15(peripherals.GPIO18)
            .with_dc(peripherals.GPIO43)
            .with_wrx(peripherals.GPIO2);
        // 2 bytes per pixel, 240 pixels per line, 4 lines.
        let buf1 = dma_tx_buffer!(480 * 4).unwrap();
        let buf2 = dma_tx_buffer!(480 * 4).unwrap();
        let writer = Writer::new(bus, buf1, buf2);
        Display::new(writer).unwrap()
    };

    println!("initializing SPIs...");
    let sd_spi = {
        let sclk = peripherals.GPIO6;
        let miso = peripherals.GPIO7;
        let mosi = peripherals.GPIO5;
        let cs = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
        let pwr = peripherals.GPIO16;
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
        let uart_config = esp_hal::uart::Config::default().with_baudrate(921_600);
        Uart::new(peripherals.UART1, uart_config)
            .unwrap()
            .with_rx(peripherals.GPIO38)
            .with_tx(peripherals.GPIO39)
    };

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    _ = usb_serial.write_byte_nb(0x00);

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("reading OTA state...");
    let mut flash = FlashStorage::new(peripherals.FLASH);
    let serial_number = read_serial(&mut flash);
    let mut pt_buf = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let mut ota = OtaUpdater::new(&mut flash, &mut pt_buf).unwrap();
    let part = ota.ota_data().unwrap().current_app_partition().unwrap();
    let main_partition = match part {
        AppPartitionSubType::Factory => 0,
        AppPartitionSubType::Ota0 => 1,
        AppPartitionSubType::Ota1 => 2,
        _ => unreachable!(),
    };

    println!("initializing device...");
    let rng = Rng::new();
    let mut device = DeviceImpl::new(sd_spi, io_uart, usb_serial, rng)?;
    let (io_version, io_partition) = device.get_io_chip_info().unwrap_or_default();
    let mut config = RuntimeConfig {
        id: None,
        device,
        display,
        net_handler: NetHandler::None,
    };
    config.apply_settings();

    println!("reading device info...");
    config.save_device_info(DeviceInfo {
        model: 2,
        serial: serial_number,
        main_version: get_firmware_version(),
        io_version,
        main_partition,
        io_partition,
    });

    println!("running...");
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

fn get_firmware_version() -> (u8, u8, u8) {
    let raw = env!("CARGO_PKG_VERSION");
    let mut iter = raw.split('.');
    let major: u8 = iter.next().unwrap().parse().unwrap();
    let minor: u8 = iter.next().unwrap().parse().unwrap();
    let patch: u8 = iter.next().unwrap().parse().unwrap();
    (major, minor, patch)
}

fn read_serial(flash: &mut FlashStorage) -> u32 {
    let mut buf = [0, 0, 0, 0];
    _ = flash.read(0x10000, &mut buf);
    u32::from_le_bytes(buf)
}
