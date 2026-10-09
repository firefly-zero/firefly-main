#![no_std]
#![no_main]
extern crate alloc;

use anyhow::Result;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::peripherals::Peripherals;
use esp_hal::system::software_reset;
use esp_hal::xtensa_lx_rt::entry;
use esp_println::println;
use firefly_main::*;

// https://github.com/esp-rs/espflash/issues/927
// https://github.com/esp-rs/esp-hal/releases/tag/esp-hal-v1.0.0-rc.0
esp_bootloader_esp_idf::esp_app_desc!();

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(size: 280 * 1024);
    println!("initializing peripherals...");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let res = run(peripherals);
    match res {
        Ok(()) => println!("unexpected exit"),
        Err(err) => println!("fatal error: {}", ErrorChain(err)),
    }

    // If the code fails, restart the chip.
    let delay = Delay::new();
    delay.delay(esp_hal::time::Duration::from_millis(500));
    software_reset();
}

fn run(peripherals: Peripherals) -> Result<()> {
    if cfg!(feature = "v3") {
        run_v3(peripherals)?;
    } else if cfg!(feature = "v2") {
        run_v2(peripherals)?;
    } else {
        panic!("unsupported hardware version");
    }
    Ok(())
}
