use crate::*;
use anyhow::{bail, Context, Result};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::AppPartitionSubType;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::lcd_cam::lcd::i8080::I8080;
use esp_hal::lcd_cam::LcdCam;
use esp_hal::peripherals::{Peripherals, CPU_CTRL};
use esp_hal::rtc_cntl::sleep::{LowPower, RtcSleepConfig};
use esp_hal::spi::master::Spi;
use esp_hal::system::{CpuControl, Stack};
use esp_hal::time::Rate;
use esp_hal::uart::{Uart, WakeupConfig};
use esp_hal::usb::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{assign_resources, dma_tx_buffer, dma_tx_stream_buffer};
use esp_println::println;
use esp_storage::FlashStorage;
use firefly_hal::DeviceImpl;
use firefly_runtime::{audio, DeviceInfo, NetHandler, NextApp, Runtime, RuntimeConfig};

static mut AUDIO_STACK: Stack<4096> = Stack::new();

assign_resources! {
    Resources<'d> {
        // I2S pins for playing audio on speakers.
        audio: AudioResources<'d> {
            i2s: I2S0,
            dma: DMA_CH1,
            ws: GPIO17,   // LRCK/WS:     Word Select / Left-Right Clock
            bclk: GPIO8,  // BCLK/SCK:    Bit Clock
            dout: GPIO18, // DATA:        Serial Data
            mclk: GPIO3,  // MCLK/SYSCLK: Master Clock
        },
        sd_card: SdCardResources<'d> {
            sclk: GPIO9,
            miso: GPIO46,
            mosi: GPIO10,
            cs: GPIO11,
            spi: SPI2,
        },
        io_uart: IoUartResources<'d> {
            uart: UART1,
            rx: GPIO15,
            tx: GPIO7,
        },
        display: DisplayResources<'d> {
            d0: GPIO12,
            d1: GPIO13,
            d2: GPIO14,
            d3: GPIO21,
            d4: GPIO47,
            d5: GPIO48,
            d6: GPIO45,
            d7: GPIO38,
            d8: GPIO39,
            d9: GPIO40,
            d10: GPIO41,
            d11: GPIO42,
            d12: GPIO44,
            d13: GPIO43,
            d14: GPIO2,
            d15: GPIO1,
            dc: GPIO5,
            wrx: GPIO4,
            dma: DMA_CH0,
            cam: LCD_CAM,
        }
    }
}

pub fn run_v3(peripherals: Peripherals) -> Result<()> {
    let resources = split_resources!(peripherals);
    init_psram_heap(peripherals.PSRAM);

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing SPIs...");
    let sd_spi = create_sd_spi(resources.sd_card)?;
    let io_uart = create_io_uart(resources.io_uart)?;
    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    _ = usb_serial.write_byte_nb(0x00);

    println!("reading OTA state...");
    let mut flash = FlashStorage::new(peripherals.FLASH);
    let serial_number = read_serial(&mut flash);
    let main_partition = get_partition(&mut flash)?;

    println!("initializing device...");
    let mut device = DeviceImpl::new(sd_spi, io_uart, usb_serial, flash).context("init device")?;
    let (io_version, io_partition) = device.get_io_chip_info().unwrap_or_default();
    let mut config = RuntimeConfig {
        next: NextApp::Launcher,
        device,
        display: create_display(resources.display)?,
        net_handler: NetHandler::None,
    };
    config.apply_settings();

    println!("reading device info...");
    config.save_device_info(DeviceInfo {
        model: 2,
        serial: serial_number,
        main_version: get_firmware_version(),
        io_version,
        main_partition,
        io_partition,
    });

    start_audio_process(peripherals.CPU_CTRL, resources.audio)?;

    println!("running...");
    loop {
        let mut runtime = wrap(Runtime::new(config)).context("init runtime")?;
        wrap(runtime.start()).context("start runtime")?;
        loop {
            let exit = wrap(runtime.update()).context("run update cycle")?;
            // Exit requested. Finalize runtime and get ownership of the device back.
            if exit {
                config = wrap(runtime.finalize()).context("finalize runtime")?;
                if config.next == NextApp::PowerOff {
                    config.finalize();
                    LowPower::new(peripherals.LPWR).sleep_deep(RtcSleepConfig::deep());
                }
                // Reload system settings in case they were changed by sys.settings.
                config.apply_settings();
                break;
            }
        }
    }
}

#[inline(never)]
fn start_audio_process(cpu_ctrl: CPU_CTRL, audio: AudioResources) -> Result<()> {
    let mut cpus = CpuControl::new(cpu_ctrl);
    #[expect(static_mut_refs)]
    let stack = unsafe { &mut AUDIO_STACK };
    match cpus.start_app_core(stack, || audio_thread(audio)) {
        Ok(guard) => core::mem::forget(guard),
        Err(_) => bail!("cannot start audio processor, app core is already running"),
    };
    Ok(())
}

#[inline(never)]
fn create_display(pins: DisplayResources<'_>) -> Result<Display<'_>> {
    let lcd_cam = LcdCam::new(pins.cam);
    let config = esp_hal::lcd_cam::lcd::i8080::Config::default();
    let Ok(bus) = I8080::new(lcd_cam.lcd, pins.dma, config) else {
        bail!("failed to create I8080 bus")
    };
    let bus = bus
        .with_data0(pins.d0)
        .with_data1(pins.d1)
        .with_data2(pins.d2)
        .with_data3(pins.d3)
        .with_data4(pins.d4)
        .with_data5(pins.d5)
        .with_data6(pins.d6)
        .with_data7(pins.d7)
        .with_data8(pins.d8)
        .with_data9(pins.d9)
        .with_data10(pins.d10)
        .with_data11(pins.d11)
        .with_data12(pins.d12)
        .with_data13(pins.d13)
        .with_data14(pins.d14)
        .with_data15(pins.d15)
        .with_dc(pins.dc)
        .with_wrx(pins.wrx);
    // 2 bytes per pixel, 240 pixels per line, 4 lines.
    let buf1 = dma_tx_buffer!(480 * 4)?;
    let buf2 = dma_tx_buffer!(480 * 4)?;
    let writer = Writer::new(bus, buf1, buf2);
    let display = Display::new(writer)?;
    Ok(display)
}

#[inline(never)]
fn create_sd_spi(
    pins: SdCardResources<'_>,
) -> Result<ExclusiveDevice<Spi<'_, esp_hal::Blocking>, Output<'_>, Delay>> {
    let cs = Output::new(pins.cs, Level::High, OutputConfig::default());
    let spi_config = esp_hal::spi::master::Config::default().with_frequency(Rate::from_mhz(4));
    let spi = Spi::new(pins.spi, spi_config).context("create SPI driver")?;
    let spi = spi
        .with_sck(pins.sclk)
        .with_miso(pins.miso)
        .with_mosi(pins.mosi);
    ExclusiveDevice::new(spi, cs, Delay::new()).context("create SPI device")
}

#[inline(never)]
fn create_io_uart(pins: IoUartResources<'_>) -> Result<Uart<'_, esp_hal::Blocking>> {
    let mut io_uart = {
        let uart_config = esp_hal::uart::Config::default().with_baudrate(921_600);
        let uart = Uart::new(pins.uart, uart_config).context("create UART")?;
        uart.with_rx(pins.rx).with_tx(pins.tx)
    };
    io_uart
        .enable_wakeup(&WakeupConfig::default())
        .context("enable wakeup on IO")?;
    Ok(io_uart)
}

// Rust compile aggressively inlines all functions to squeeze out the best performance.
// This, however, can leave on stack of the main function some things that we don't
// need anymore. To prevent this, we move as many things as possible into separate
// functions and forbid their inlining.
#[inline(never)]
fn get_partition(flash: &mut FlashStorage<'_>) -> Result<u8> {
    let mut pt_buf = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let mut ota = OtaUpdater::new(flash, &mut pt_buf)?;
    let part = ota.ota_data()?.current_app_partition()?;
    let part = match part {
        AppPartitionSubType::Factory => 0,
        AppPartitionSubType::Ota0 => 1,
        AppPartitionSubType::Ota1 => 2,
        _ => unreachable!(),
    };
    Ok(part)
}

fn get_firmware_version() -> (u8, u8, u8) {
    let major: u8 = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
    let minor: u8 = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
    let patch: u8 = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();
    (major, minor, patch)
}

fn read_serial(flash: &mut FlashStorage) -> u32 {
    let mut buf = [0, 0, 0, 0];
    _ = flash.read(0x10000, &mut buf);
    u32::from_le_bytes(buf)
}

fn wrap<T, E: core::fmt::Display>(r: Result<T, E>) -> Result<T> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => bail!("{e}"),
    }
}

fn audio_thread(pins: AudioResources) {
    use esp_hal::i2s::master::*;

    let config = TdmConfig::new_tdm_philips()
        .with_sample_rate(Rate::from_hz(44100))
        .with_data_format(DataFormat::Data16Channel16)
        .with_channels(Channels::STEREO);
    let i2s = I2s::new(pins.i2s, pins.dma, config)
        .unwrap()
        .with_mclk(pins.mclk);

    let mut tx = i2s
        .i2s_tx
        .with_bclk(pins.bclk)
        .with_ws(pins.ws)
        .with_dout(pins.dout)
        .build();

    let mut buffer = dma_tx_stream_buffer!(4096, 1024);
    loop {
        buffer.push_with(fill_audio);
        let mut transaction = tx.write(buffer).unwrap();
        while !transaction.is_done() {
            if transaction.available_bytes() >= 4 {
                transaction.push_with(fill_audio);
            }
        }
        (tx, buffer) = transaction.stop();
    }
}

fn fill_audio(buf: &mut [u8]) -> usize {
    let len = buf.len();
    let ptr = buf.as_ptr() as *mut i16;
    let buf: &mut [i16] = unsafe { core::slice::from_raw_parts_mut(ptr, len / 2) };
    audio::exec_external(|manager| {
        manager.write(buf);
    });
    len / 2 * 2
}
