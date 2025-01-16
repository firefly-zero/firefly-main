#![no_std]
#![no_main]
extern crate alloc;

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_tx_buffer,
    gpio::{Level, Output},
    lcd_cam::{
        lcd::i8080::{TxSixteenBits, I8080},
        LcdCam,
    },
    prelude::*,
    rng::Rng,
    spi::master::Spi,
    uart::Uart,
};
use esp_println::println;
use firefly_hal::DeviceImpl;
use firefly_main::*;
use firefly_runtime::{FullID, NetHandler, Runtime, RuntimeConfig};

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(200 * 1024);
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
        let mut rst = Output::new(rst, Level::Low);
        rst.set_high();

        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.channel0.configure(false, DmaPriority::Priority0);
        let bus = I8080::new(
            lcd_cam.lcd,
            dma_channel.tx,
            tx_pins,
            20.MHz(),
            esp_hal::lcd_cam::lcd::i8080::Config::default(),
        )
        .with_cs(peripherals.GPIO1)
        .with_ctrl_pins(peripherals.GPIO8, peripherals.GPIO3);
        let buf = dma_tx_buffer!(480 * 3 * 10).unwrap(); // 10 lines
        let commander = ili9488::Commander::new(bus, buf);
        let mut display = ili9488::Display::<ili9488::Rgb888>::new(commander).unwrap();
        let orientation = ili9488::Orientation {
            flip_x: true,
            flip_y: true,
            rotate: true,
        };
        display.set_orientation(orientation).unwrap();
        display
        // ili9488::Scaler {
        //     display,
        //     x: 2,
        //     y: 1,
        // }
    };

    println!("initializing SPIs...");
    let sd_spi = {
        let sclk = peripherals.GPIO15;
        let miso = peripherals.GPIO7;
        let mosi = peripherals.GPIO16;
        let cs = Output::new(peripherals.GPIO17, Level::High);
        let pwr = peripherals.GPIO47;
        Output::new(pwr, Level::High);
        Delay::new().delay_millis(10);

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
    let io_uart = {
        let miso = peripherals.GPIO4;
        let mosi = peripherals.GPIO5;
        Uart::new(peripherals.UART1, miso, mosi).unwrap()
    };

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing device...");
    let rng = Rng::new(peripherals.RNG);
    let device = DeviceImpl::new(sd_spi, io_uart, rng)?;
    let mut config = RuntimeConfig {
        // id: None,
        id: Some(FullID::new(
            "sys".try_into().unwrap(),
            "input-test".try_into().unwrap(),
        )),
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
