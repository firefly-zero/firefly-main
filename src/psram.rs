use esp_hal::peripherals::PSRAM;
use esp_hal::psram::Psram;

/// Initialize PSRAM and add it as a heap memory region
pub fn init_psram_heap(peripheral: PSRAM<'static>) {
    let psram_config = esp_hal::psram::PsramConfig {
        mode: esp_hal::psram::PsramMode::OctalSpi,
        ..Default::default()
    };
    let psram = Psram::new(peripheral, psram_config);
    let (start, size) = psram.raw_parts();
    let capabilities = esp_alloc::MemoryCapability::External.into();
    unsafe {
        let region = esp_alloc::HeapRegion::new(start, size, capabilities);
        esp_alloc::HEAP.add_region(region);
    }
}
