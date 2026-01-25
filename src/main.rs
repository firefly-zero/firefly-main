#![no_std]
#![no_main]
extern crate alloc;

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, delay::Delay, system::software_reset, xtensa_lx_rt::entry};
use esp_println::println;
use firefly_main::*;

/// Initialize PSRAM and add it as a heap memory region
pub(crate) fn init_psram_heap(start: *mut u8, size: usize) {
    let capabilities = esp_alloc::MemoryCapability::External.into();
    unsafe {
        let region = esp_alloc::HeapRegion::new(start, size, capabilities);
        esp_alloc::HEAP.add_region(region);
    }
}

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(size: 280 * 1024);
    println!("initializing peripherals...");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let (start, size) = esp_hal::psram::psram_raw_parts(&peripherals.PSRAM);
    init_psram_heap(start, size);

    #[cfg(not(feature = "v2"))]
    let res = run_v1(peripherals);
    #[cfg(feature = "v2")]
    let res = run_v2(peripherals);
    match res {
        Ok(()) => println!("unexpected exit"),
        Err(err) => println!("fatal error: {err}"),
    }

    // If the code fails, restart the chip.
    let delay = Delay::new();
    delay.delay(esp_hal::time::Duration::from_millis(500));
    software_reset();
}
