pub enum Error {
    Uart(esp_hal::uart::Error),
    Runtime(firefly_runtime::Error),
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
            Error::Uart(error) => write!(f, "UART error: {error:?}"),
            Error::Runtime(error) => write!(f, "runtime error: {error}"),
        }
    }
}
