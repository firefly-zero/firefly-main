#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{delay::Delay, prelude::*, timer::timg::TimerGroup, uart::Uart};
use firefly_hal::DeviceImpl;
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};
use firefly_supervisor::*;

#[entry]
fn main() -> ! {
    let res = run();
    if let Err(err) = res {
        log::error!("{err}");
    }
    log::info!("end");
    let delay = Delay::new();
    loop {
        delay.delay(500.millis());
    }
}

fn run() -> Result<(), Error> {
    esp_println::logger::init_logger_from_env();
    log::info!("creating device config...");
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    log::info!("initializing peripherals...");
    let peripherals = esp_hal::init(config);
    log::info!("initializing UART...");
    let uart = Uart::new(peripherals.UART1, peripherals.GPIO1, peripherals.GPIO2)?;
    log::info!("initializing device...");
    let device = DeviceImpl::new(uart)?;
    log::info!("initializing display...");
    let display = Display::new();
    let config = RuntimeConfig {
        id: None,
        device,
        display,
        net_handler: NetHandler::None,
    };
    log::info!("creating runtime...");
    let runtime = Runtime::new(config)?;
    log::info!("running...");
    runtime.run()?;
    Ok(())
}
