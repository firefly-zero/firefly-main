#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::prelude::*;
use firefly_hal::*;
use firefly_runtime::*;
use firefly_supervisor::*;

#[entry]
fn main() -> ! {
    let res = run();
    if let Err(err) = res {
        panic!("{err}");
    }
    panic!("end")
}

fn run() -> Result<(), Error> {
    let config = RuntimeConfig {
        id: None,
        device: DeviceImpl::new(),
        display: Display::new(),
        net_handler: NetHandler::None,
    };
    let runtime = Runtime::new(config)?;
    runtime.run()?;
    Ok(())
}
