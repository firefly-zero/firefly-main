use crate::*;
use anyhow::{bail, Context, Result};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_bootloader_esp_idf::ota_updater::OtaUpdater;
use esp_bootloader_esp_idf::partitions::AppPartitionSubType;
use esp_hal::delay::Delay;
use esp_hal::dma::DmaTxStreamBuf;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::lcd_cam::lcd::i8080::I8080;
use esp_hal::lcd_cam::LcdCam;
use esp_hal::peripherals::Peripherals;
use esp_hal::psram::Psram;
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
    }
}

pub fn run_v3(peripherals: Peripherals) -> Result<()> {
    let resources = split_resources!(peripherals);
    let psram_config = esp_hal::psram::PsramConfig {
        mode: esp_hal::psram::PsramMode::OctalSpi,
        ..Default::default()
    };
    let psram = Psram::new(peripherals.PSRAM, psram_config);
    let (start, size) = psram.raw_parts();
    init_psram_heap(start, size);

    println!("waiting for IO to start...");
    Delay::new().delay_millis(1000);

    println!("initializing display...");
    let display = {
        let lcd_cam = LcdCam::new(peripherals.LCD_CAM);
        let config = esp_hal::lcd_cam::lcd::i8080::Config::default();
        let Ok(bus) = I8080::new(lcd_cam.lcd, peripherals.DMA_CH0, config) else {
            bail!("failed to create I8080 bus")
        };
        let bus = bus
            .with_data0(peripherals.GPIO12)
            .with_data1(peripherals.GPIO13)
            .with_data2(peripherals.GPIO14)
            .with_data3(peripherals.GPIO21)
            .with_data4(peripherals.GPIO47)
            .with_data5(peripherals.GPIO48)
            .with_data6(peripherals.GPIO45)
            .with_data7(peripherals.GPIO38)
            .with_data8(peripherals.GPIO39)
            .with_data9(peripherals.GPIO40)
            .with_data10(peripherals.GPIO41)
            .with_data11(peripherals.GPIO42)
            .with_data12(peripherals.GPIO44)
            .with_data13(peripherals.GPIO43)
            .with_data14(peripherals.GPIO2)
            .with_data15(peripherals.GPIO1)
            .with_dc(peripherals.GPIO5)
            .with_wrx(peripherals.GPIO4);
        // 2 bytes per pixel, 240 pixels per line, 4 lines.
        let buf1 = dma_tx_buffer!(480 * 4)?;
        let buf2 = dma_tx_buffer!(480 * 4)?;
        let writer = Writer::new(bus, buf1, buf2);
        Display::new(writer)?
    };

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
        display,
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

    {
        let mut cpus = CpuControl::new(peripherals.CPU_CTRL);
        #[expect(static_mut_refs)]
        let stack = unsafe { &mut AUDIO_STACK };
        let buffer = dma_tx_stream_buffer!(4092, 1024);
        match cpus.start_app_core(stack, || audio_thread(resources.audio, buffer)) {
            Ok(guard) => core::mem::forget(guard),
            Err(_) => bail!("cannot start audio processor, app core is already running"),
        };
    }

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
                break;
            }
        }
    }
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

fn audio_thread(pins: AudioResources, mut buffer: DmaTxStreamBuf) {
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

    buffer.push_with(fill_audio);
    loop {
        let mut transaction = tx.write(buffer).unwrap();
        transaction.push_with(fill_audio);
        let res;
        (res, tx, buffer) = transaction.wait();
        res.unwrap();
    }
}

fn fill_audio(buf: &mut [u8]) -> usize {
    let len = buf.len();
    let ptr = buf.as_ptr() as *mut i16;
    let buf: &mut [i16] = unsafe { core::slice::from_raw_parts_mut(ptr, len / 2) };
    audio::exec_external(|manager| {
        manager.write(buf);
    });
    buf.len()
}
