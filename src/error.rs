pub enum Error {
    Uart(&'static str),
    Runtime(firefly_runtime::Error),
    Network(firefly_hal::NetworkError),
    Display,
    Pin,
}

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
