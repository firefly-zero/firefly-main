/// Initialize PSRAM and add it as a heap memory region
#[expect(clippy::not_unsafe_ptr_arg_deref)]
pub fn init_psram_heap(start: *mut u8, size: usize) {
    let capabilities = esp_alloc::MemoryCapability::External.into();
    unsafe {
        let region = esp_alloc::HeapRegion::new(start, size, capabilities);
        esp_alloc::HEAP.add_region(region);
    }
}
