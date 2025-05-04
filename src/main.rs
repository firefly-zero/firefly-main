#![no_std]
#![no_main]
extern crate alloc;

use core::cell::RefCell;
use core::ptr::addr_of_mut;

use alloc::rc::Rc;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::otg_fs::{Usb, UsbBus};
use esp_hal::time::{Duration, Rate};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::PeriodicTimer;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma_tx_buffer,
    gpio::{Level, Output, OutputConfig},
    handler,
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
use firefly_runtime::{NetHandler, Runtime, RuntimeConfig};
use usb_device::bus::UsbBusAllocator;
use usb_device::device::UsbDevice;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

/// Initialize PSRAM and add it as a heap memory region
fn init_psram_heap(start: *mut u8, size: usize) {
    let capabilities = esp_alloc::MemoryCapability::External.into();
    unsafe {
        let region = esp_alloc::HeapRegion::new(start, size, capabilities);
        esp_alloc::HEAP.add_region(region);
    }
}

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

type UsbBusAlloc = UsbBusAllocator<UsbBus<Usb<'static>>>;
static mut USB_BUS: Option<UsbBusAlloc> = None;
type UsbSerial = SerialPort<'static, UsbBus<Usb<'static>>>;
static mut USB_SERIAL: Option<Rc<RefCell<UsbSerial>>> = None;
static mut USB_DEVICE: Option<UsbDevice<'static, UsbBus<Usb<'static>>>> = None;

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(280 * 1024);
    let res = run();
    if let Err(err) = res {
        println!("ERROR: {err}");
    }
    println!("end");
    let delay = Delay::new();
    loop {
        delay.delay(esp_hal::time::Duration::from_millis(500u64));
    }
}

fn run() -> Result<(), Error> {
    println!("creating device config...");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    println!("initializing peripherals...");
    let peripherals = esp_hal::init(config);
    let (start, size) = esp_hal::psram::psram_raw_parts(&peripherals.PSRAM);
    init_psram_heap(start, size);
    // println!("initializing UART...");
    // let uart = Uart::new(peripherals.UART1, peripherals.GPIO1, peripherals.GPIO2)?;

    println!("initializing display...");
    let mut display = {
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
        let mut rst = Output::new(rst, Level::Low, OutputConfig::default());
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
        Display::new(writer).unwrap()
    };
    display.clear(Rgb565::BLUE);

    println!("initializing SPIs...");
    let sd_spi = {
        let sclk = peripherals.GPIO15;
        let miso = peripherals.GPIO7;
        let mosi = peripherals.GPIO16;
        let cs = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());
        let pwr = peripherals.GPIO47;
        Output::new(pwr, Level::High, OutputConfig::default());
        Delay::new().delay_millis(10);

        let spi_config =
            esp_hal::spi::master::Config::default().with_frequency(Rate::from_khz(200u32));
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

    display.clear(Rgb565::RED);

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let usb_bus = UsbBus::new(usb, unsafe { &mut *addr_of_mut!(EP_MEMORY) });
    unsafe { USB_BUS = Some(usb_bus) };
    let usb_bus = unsafe { USB_BUS.as_ref().unwrap() };
    let mut usb_serial = SerialPort::new(usb_bus);
    // let usb_serial = Rc::new(RefCell::new(usb_serial));
    // unsafe { USB_SERIAL = Some(Rc::clone(&usb_serial)) };
    let vid_pid = UsbVidPid(0x303A, 0x3001); // Vendor: Espressif Incorporated
    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, vid_pid)
        .device_class(USB_CLASS_CDC)
        .build();
    // unsafe { USB_DEVICE = Some(usb_dev) };
    // let syst = SystemTimer::new(peripherals.SYSTIMER);
    // let mut timer = PeriodicTimer::new(syst.alarm0);
    // timer.set_interrupt_handler(handle_usb_poll);
    // timer.start(Duration::from_millis(5)).unwrap();

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing device...");
    let rng = Rng::new(peripherals.RNG);
    let device = DeviceImpl::new(sd_spi, io_uart, usb_dev, usb_serial, rng)?;
    let mut config = RuntimeConfig {
        id: None,
        device,
        display,
        net_handler: NetHandler::None,
    };
    println!("creating runtime...");
    println!("running...");
    config.display.clear(Rgb565::GREEN);
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
            runtime.display_mut().clear(Rgb565::WHITE);
        }
    }
}

// #[handler]
// fn handle_usb_poll() {
//     unsafe {
//         let serial = &mut USB_SERIAL.as_mut().unwrap();
//         let Some(serial) = Rc::get_mut(serial) else {
//             return;
//         };
//         let serial = serial.get_mut();
//         let device = USB_DEVICE.as_mut().unwrap();
//         device.poll(&mut [serial]);

//         // let pending = device.poll(&mut [serial]);
//         // if pending {
//         //     let mut buf = [0u8; 64];
//         //     _ = serial.read(&mut buf);
//         // }
//     };
// }
