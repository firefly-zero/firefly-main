pub enum Error {
    Uart(esp_hal::uart::Error),
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

impl From<esp_hal::uart::Error> for Error {
    fn from(v: esp_hal::uart::Error) -> Self {
        Self::Uart(v)
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
