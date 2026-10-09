use core::fmt::Display;

/// A wrapper for [`anyhow::Error`] that prints it as Go errors.
///
/// So, instead of:
///
/// ```text
/// read config file
///
/// Caused by:
///     No such file or directory (os error 2)
/// ```
///
/// It will print:
///
/// ```text
/// read config file: No such file or directory (os error 2).
/// ```
pub struct ErrorChain(pub anyhow::Error);

impl Display for ErrorChain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let error = &self.0;
        write!(f, "{error}")?;
        if let Some(cause) = error.source() {
            for error in anyhow::Chain::new(cause) {
                write!(f, ": {error}")?;
            }
        }
        write!(f, ".")?;
        Ok(())
    }
}

pub enum Error {
    Uart(&'static str),
    Runtime(firefly_runtime::Error),
    Network(firefly_hal::NetworkError),
    Display,
    Pin,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

impl core::error::Error for Error {}

impl From<firefly_hal::NetworkError> for Error {
    fn from(v: firefly_hal::NetworkError) -> Self {
        Self::Network(v)
    }
}

impl From<firefly_runtime::Error> for Error {
    fn from(v: firefly_runtime::Error) -> Self {
        Self::Runtime(v)
    }
}

impl From<esp_hal::uart::RxError> for Error {
    fn from(value: esp_hal::uart::RxError) -> Self {
        let msg = match value {
            esp_hal::uart::RxError::FifoOverflowed => "RX FIFO overflowed",
            esp_hal::uart::RxError::GlitchOccurred => "glitch on RX line",
            esp_hal::uart::RxError::FrameFormatViolated => "framing error on RX line",
            esp_hal::uart::RxError::ParityMismatch => "parity error on RX line",
            _ => "unknown RX error",
        };
        Self::Uart(msg)
    }
}

impl From<esp_hal::uart::TxError> for Error {
    fn from(_: esp_hal::uart::TxError) -> Self {
        Self::Uart("unknown TX error")
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Uart(error) => write!(f, "UART error: {error:?}"),
            Self::Runtime(error) => write!(f, "runtime error: {error}"),
            Self::Network(error) => write!(f, "network error: {error}"),
            Self::Display => write!(f, "display error"),
            Self::Pin => write!(f, "pin error"),
        }
    }
}
