#![no_std]
#![no_main]
extern crate alloc;

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma_tx_buffer,
    gpio::{Level, Output},
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
use firefly_runtime::{FullID, NetHandler, Runtime, RuntimeConfig};
use fugit::{ExtU64, RateExtU32};

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
        delay.delay(500u64.millis());
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
        Display3::new(writer).unwrap()
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

        let mut spi_config = esp_hal::spi::master::Config::default();
        spi_config.frequency = 200u32.kHz();
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

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing device...");
    let rng = Rng::new(peripherals.RNG);
    let device = DeviceImpl::new(sd_spi, io_uart, rng)?;
    let mut config = RuntimeConfig {
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
