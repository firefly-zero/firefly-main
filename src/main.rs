#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;
use embedded_hal_bus::spi::{ExclusiveDevice, RefCellDevice};
use embedded_sdmmc::SdCard;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority},
    gpio::{Level, Output},
    prelude::*,
    spi::master::Spi,
};
use esp_println::println;
use firefly_hal::DeviceImpl;
use firefly_runtime::{FullID, NetHandler, Runtime, RuntimeConfig};
use firefly_supervisor::*;

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
    let cs = peripherals.GPIO5;
    let miso = peripherals.GPIO2;
    let dc = Output::new(peripherals.GPIO4, Level::Low);
    let mut gpio_backlight = Output::new(peripherals.GPIO45, Level::Low);
    let rst = Output::new(peripherals.GPIO48, Level::Low);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let spi = Spi::new_with_config(
        peripherals.SPI3,
        esp_hal::spi::master::Config {
            frequency: 40u32.MHz(),
            ..esp_hal::spi::master::Config::default()
        },
    )
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .with_cs(cs)
    .with_dma(dma_channel.configure(false, DmaPriority::Priority0));
    let mut delay = Delay::new();
    gpio_backlight.set_high();

    let di = new_no_cs(240 * 320 * 2, spi, dc);

    println!("initializing display...");
    let display = mipidsi::Builder::new(mipidsi::models::ILI9341Rgb565, di)
        .display_size(240, 320)
        .orientation(
            mipidsi::options::Orientation::new()
                .rotate(mipidsi::options::Rotation::Deg270)
                .flip_horizontal(),
        )
        .color_order(mipidsi::options::ColorOrder::Bgr)
        .reset_pin(rst)
        .init(&mut delay)?;

    let sclk = peripherals.GPIO13;
    let miso = peripherals.GPIO12;
    let mosi = peripherals.GPIO11;
    let cs = Output::new(peripherals.GPIO6, Level::High);

    println!("initializing SD card...");
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
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let sdcard = SdCard::new(spi_device, delay);
    if let Some(card_type) = sdcard.get_card_type() {
        println!("SD | card type: {card_type:?}");
    }
    println!("SD | num bytes: {:?}", sdcard.num_bytes());

    println!("initializing device...");
    let device = DeviceImpl::new(sdcard)?;
    let config = RuntimeConfig {
        // id: None,
        id: Some(FullID::new(
            "lux".try_into().unwrap(),
            "snek".try_into().unwrap(),
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
