#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;

use display_interface_spi::SPIInterface;
use embedded_hal_bus::spi::RefCellDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority},
    gpio::{Level, Output},
    prelude::*,
    spi::{master::Spi, SpiMode},
};
use esp_println::println;
use firefly_hal::DeviceImpl;
use firefly_runtime::{FullID, NetHandler, Runtime, RuntimeConfig};
use firefly_supervisor::*;
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(190 * 1024);
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

    let sclk = peripherals.GPIO36;
    let mosi = peripherals.GPIO35;
    // let cs = peripherals.GPIO5;
    // let miso = peripherals.GPIO2;
    let dc = Output::new(peripherals.GPIO4, Level::Low);
    // let mut gpio_backlight = Output::new(peripherals.GPIO45, Level::Low);
    let rst = Output::new(peripherals.GPIO48, Level::Low);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let spi = Spi::new_with_config(
        peripherals.SPI2,
        esp_hal::spi::master::Config {
            frequency: 40u32.MHz(),
            ..esp_hal::spi::master::Config::default()
        },
    )
    .with_sck(sclk)
    .with_mosi(mosi);
    // .with_miso(miso)
    // .with_cs(cs)
    // .with_dma(dma_channel.configure(false, DmaPriority::Priority0));
    let mut delay = Delay::new();
    // gpio_backlight.set_high();

    // let di = SPIInterface::new(spi, dc);
    // let di = new_no_cs(240 * 320 * 2, spi, dc);

    // ESP32-S3-BOX display initialization workaround: Wait for the display to power up.
    // If delay is 250ms, picture will be fuzzy.
    // If there is no delay, display is blank
    delay.delay_millis(500u32);

    let spi_bus = RefCell::new(spi);
    let cs = Output::new(peripherals.GPIO5, Level::High);
    let spi_device = RefCellDevice::new_no_delay(&spi_bus, cs).unwrap();
    let spi_iface = SPIInterface::new(spi_device, dc);

    println!("initializing display...");
    let display = Ili9341::new(
        spi_iface,
        rst,
        &mut delay,
        Orientation::Landscape,
        DisplaySize240x320,
    )
    .unwrap();

    println!("initializing device...");
    let device = DeviceImpl::new()?;
    let config = RuntimeConfig {
        // id: None,
        id: Some(FullID::new(
            "demo".try_into().unwrap(),
            "go-triangle".try_into().unwrap(),
        )),
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
