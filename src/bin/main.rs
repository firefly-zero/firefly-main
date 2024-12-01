#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{delay::Delay, prelude::*};
use esp_println::println;
use firefly_hal::DeviceImpl;
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};
use firefly_supervisor::*;

#[entry]
fn main() -> ! {
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
    println!("initializing device...");
    let device = DeviceImpl::new()?;
    println!("initializing display...");
    let display = Display::new();
    let config = RuntimeConfig {
        id: None,
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
