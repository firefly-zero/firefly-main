#![no_std]
#![no_main]
extern crate alloc;

use esp_backtrace as _;
use esp_hal::{delay::Delay, system::software_reset, xtensa_lx_rt::entry};
use esp_println::println;
use firefly_main::*;

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(size: 280 * 1024);
    #[cfg(not(feature = "v2"))]
    let res = run_v1();
    #[cfg(feature = "v2")]
    let res = run_v2();
    match res {
        Ok(()) => println!("unexpected exit"),
        Err(err) => println!("fatal error: {err}"),
    }

    // If the code fails, restart the chip.
    let delay = Delay::new();
    delay.delay(esp_hal::time::Duration::from_millis(500u64));
    software_reset();
}
