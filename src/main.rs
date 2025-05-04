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
    lcd_cam::{
        lcd::i8080::{TxSixteenBits, I8080},
        LcdCam,
    },
    rng::Rng,
    spi::master::Spi,
    uart::Uart,
    xtensa_lx_rt::entry,
};
use esp_println::println;
use firefly_hal::DeviceImpl;
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
    esp_alloc::heap_allocator!(280 * 1024);
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
    let display = {
        let tx_pins = TxSixteenBits::new(
            peripherals.GPIO9,
            peripherals.GPIO10,
            peripherals.GPIO11,
            peripherals.GPIO12,
            peripherals.GPIO13,
            peripherals.GPIO14,
            peripherals.GPIO21,
            peripherals.GPIO45,
            peripherals.GPIO38,
            peripherals.GPIO39,
            peripherals.GPIO40,
            peripherals.GPIO41,
            peripherals.GPIO42,
            peripherals.GPIO44,
            peripherals.GPIO43,
            peripherals.GPIO2,
        );

        // hardware reset
        let rst = peripherals.GPIO46;
        let mut rst = Output::new(rst, Level::Low, OutputConfig::default());
        rst.set_high();

        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let bus = I8080::new(
            lcd_cam.lcd,
            peripherals.DMA_CH0,
            tx_pins,
            esp_hal::lcd_cam::lcd::i8080::Config::default(),
        )
        .unwrap()
        .with_cs(peripherals.GPIO1)
        .with_ctrl_pins(peripherals.GPIO8, peripherals.GPIO3);
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

        let spi_config =
            esp_hal::spi::master::Config::default().with_frequency(Rate::from_khz(200u32));
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
        let uart_config = esp_hal::uart::Config::default();
        Uart::new(peripherals.UART1, uart_config)
            .unwrap()
            .with_rx(miso)
            .with_tx(mosi)
    };
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing device...");
    let rng = Rng::new(peripherals.RNG);
    let device = DeviceImpl::new(sd_spi, io_uart, usb_serial, rng)?;
    let mut config = RuntimeConfig {
        id: None,
        device,
        display,
        net_handler: NetHandler::None,
    };
    println!("creating runtime...");
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

// #[handler]
// fn handle_usb_poll() {
//     unsafe {
//         let serial = &mut USB_SERIAL.as_mut().unwrap();
//         let Some(serial) = Rc::get_mut(serial) else {
//             return;
//         };
//         let serial = serial.get_mut();
//         let device = USB_DEVICE.as_mut().unwrap();
//         device.poll(&mut [serial]);

//         // let pending = device.poll(&mut [serial]);
//         // if pending {
//         //     let mut buf = [0u8; 64];
//         //     _ = serial.read(&mut buf);
//         // }
//     };
// }
