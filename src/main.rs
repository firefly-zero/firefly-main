#![no_std]
#![no_main]
extern crate alloc;

use esp_backtrace as _;
use esp_hal::{delay::Delay, xtensa_lx_rt::entry};
use esp_println::println;
use firefly_main::*;

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(size: 280 * 1024);
    #[cfg(not(feature = "v2"))]
    let res = run_v1();
    #[cfg(feature = "v2")]
    let res = run_v2();
    if let Err(err) = res {
        println!("ERROR: {err}");
    }
    println!("end");
    let delay = Delay::new();
    loop {
        delay.delay(esp_hal::time::Duration::from_millis(500u64));
    }
}
