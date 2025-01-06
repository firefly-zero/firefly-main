#![no_std]
#![no_main]
extern crate alloc;

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority},
    gpio::{Level, Output},
    lcd_cam::{
        lcd::i8080::{TxSixteenBits, I8080},
        LcdCam,
    },
    prelude::*,
    spi::master::Spi,
};
use esp_println::println;
use firefly_hal::DeviceImpl;
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};
use firefly_supervisor::*;

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(300 * 1024);
    let res = run();
    if let Err(err) = res {
        println!("ERROR: {err}");
    }
    println!("end");
    let delay = Delay::new();
    loop {
        delay.delay(500.millis());
    }
}

fn run() -> Result<(), Error> {
    println!("creating device config...");
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    println!("initializing peripherals...");
    let peripherals = esp_hal::init(config);
    // println!("initializing UART...");
    // let uart = Uart::new(peripherals.UART1, peripherals.GPIO1, peripherals.GPIO2)?;

    let mut delay = Delay::new();

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
        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.channel0.configure(false, DmaPriority::Priority0);
        let mut i8080 = I8080::new(
            lcd_cam.lcd,
            dma_channel.tx,
            tx_pins,
            20.MHz(),
            esp_hal::lcd_cam::lcd::i8080::Config::default(),
        )
        .with_ctrl_pins(peripherals.GPIO1, peripherals.GPIO15);
        todo!()
    };

    println!("initializing SPIs...");
    let sd_spi = {
        let sclk = peripherals.GPIO15;
        let miso = peripherals.GPIO7;
        let mosi = peripherals.GPIO16;
        let cs = Output::new(peripherals.GPIO17, Level::High);

        let spi = Spi::new_with_config(
            peripherals.SPI2,
            esp_hal::spi::master::Config {
                frequency: 200u32.kHz(),
                ..esp_hal::spi::master::Config::default()
            },
        )
        .with_sck(sclk)
        .with_miso(miso)
        .with_mosi(mosi);
        ExclusiveDevice::new(spi, cs, Delay::new()).unwrap()
    };
    let io_spi = {
        let sclk = peripherals.GPIO6;
        let miso = peripherals.GPIO5;
        let mosi = peripherals.GPIO4;
        let spi = Spi::new_with_config(
            peripherals.SPI3,
            esp_hal::spi::master::Config {
                frequency: 200u32.kHz(),
                ..esp_hal::spi::master::Config::default()
            },
        )
        .with_sck(sclk)
        .with_miso(miso)
        .with_mosi(mosi);
        ExclusiveDevice::new(spi, cs, Delay::new()).unwrap()
    };

    println!("initializing device...");
    let device = DeviceImpl::new(sd_spi, io_spi)?;
    let config = RuntimeConfig {
        id: None,
        // id: Some(FullID::new(
        //     "lux".try_into().unwrap(),
        //     "snek".try_into().unwrap(),
        // )),
        device,
        display,
        net_handler: NetHandler::None,
    };
    println!("creating runtime...");
    let runtime = Runtime::new(config)?;
    println!("running...");
    runtime.run()?;
    Ok(())
}
