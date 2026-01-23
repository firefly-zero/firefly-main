#![no_std]
#![no_main]
extern crate alloc;

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::time::Rate;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma_tx_buffer,
    gpio::{Level, Output, OutputConfig},
    lcd_cam::{lcd::i8080::I8080, LcdCam},
    rng::Rng,
    spi::master::Spi,
    uart::Uart,
    xtensa_lx_rt::entry,
};
use esp_println::println;
use firefly_hal::{Device, DeviceImpl};
use firefly_main::*;
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};

/// Initialize PSRAM and add it as a heap memory region
fn init_psram_heap(start: *mut u8, size: usize) {
    let capabilities = esp_alloc::MemoryCapability::External.into();
    unsafe {
        let region = esp_alloc::HeapRegion::new(start, size, capabilities);
        esp_alloc::HEAP.add_region(region);
    }
}

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(size: 280 * 1024);
    let res = run();
    if let Err(err) = res {
        println!("ERROR: {err}");
    }
    println!("end");
    let delay = Delay::new();
    loop {
        delay.delay(esp_hal::time::Duration::from_millis(500u64));
    }
}

fn run() -> Result<(), Error> {
    println!("creating device config...");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    println!("initializing peripherals...");
    let peripherals = esp_hal::init(config);
    let (start, size) = esp_hal::psram::psram_raw_parts(&peripherals.PSRAM);
    init_psram_heap(start, size);

    println!("initializing display...");
    #[cfg(not(feature = "v2"))]
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

    #[cfg(feature = "v2")]
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
    #[cfg(not(feature = "v2"))]
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

    #[cfg(feature = "v2")]
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

    #[cfg(not(feature = "v2"))]
    let io_uart = {
        let miso = peripherals.GPIO4;
        let mosi = peripherals.GPIO5;
        let uart_config = esp_hal::uart::Config::default().with_baudrate(921_600);
        Uart::new(peripherals.UART1, uart_config)
            .unwrap()
            .with_rx(miso)
            .with_tx(mosi)
    };
    #[cfg(feature = "v2")]
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
