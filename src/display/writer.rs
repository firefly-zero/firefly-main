use esp_hal::{
    dma::{DmaError, DmaTxBuf},
    lcd_cam::lcd::i8080::{Command, I8080Transfer, I8080},
    Blocking,
};

type Future<'a> = I8080Transfer<'a, DmaTxBuf, Blocking>;

#[derive(Debug)]
pub enum Error {
    Dma(DmaError),
    OutOfBuffers,
    TooManyBuffers,
    BusIsBusy,
}

impl From<DmaError> for Error {
    fn from(v: DmaError) -> Self {
        Self::Dma(v)
    }
}

/// Either an i8080 bus, a future owning the bus, or None.
#[derive(Default)]
enum Channel<'a> {
    Bus(I8080<'a, Blocking>),
    Future(Future<'a>),
    /// None is set in the brief period between when we got the bus ownership
    /// and when we called `send` on it.
    #[default]
    None,
}

pub struct Writer<'a> {
    channel: Channel<'a>,
    /// Pool of DMA buffers.
    buffers: [Option<DmaTxBuf>; 2],
}

impl<'a> Writer<'a> {
    pub fn new(bus: I8080<'a, Blocking>, buf1: DmaTxBuf, buf2: DmaTxBuf) -> Self {
        Self {
            channel: Channel::Bus(bus),
            buffers: [Some(buf1), Some(buf2)],
        }
    }

    pub fn take_buffer(&mut self) -> Result<DmaTxBuf, Error> {
        if let Some(buf) = self.try_take_buffer()? {
            return Ok(buf);
        }
        self.wait()?;
        if let Some(buf) = self.try_take_buffer()? {
            return Ok(buf);
        }
        Err(Error::OutOfBuffers)
    }

    /// Get ownership of the first available buffer
    fn try_take_buffer(&mut self) -> Result<Option<DmaTxBuf>, Error> {
        // Try to find an available buffer.
        for maybe_buf in self.buffers.iter_mut() {
            if let Some(buf) = maybe_buf.take() {
                return Ok(Some(buf));
            }
        }
        Ok(None)
    }

    /// Find a pending future and await it.
    pub fn wait(&mut self) -> Result<(), Error> {
        match core::mem::take(&mut self.channel) {
            Channel::Future(fut) => {
                let (res, bus, buf) = fut.wait();
                self.channel = Channel::Bus(bus);
                self.put_buffer(buf)?;
                res?;
                Ok(())
            }
            Channel::Bus(bus) => {
                self.channel = Channel::Bus(bus);
                Ok(())
            }
            Channel::None => Err(Error::BusIsBusy),
        }
    }

    /// Return the given buffer back into the pool.
    ///
    /// Called when the future owning the buffer is resolved.
    pub fn put_buffer(&mut self, buf: DmaTxBuf) -> Result<(), Error> {
        for maybe_buf in self.buffers.iter_mut() {
            if maybe_buf.is_none() {
                *maybe_buf = Some(buf);
                return Ok(());
            }
        }
        Err(Error::TooManyBuffers)
    }

    /// Send a short command using 8-bit transfer.
    pub fn send_cmd<const N: usize>(&mut self, cmd: u8, params: [u8; N]) -> Result<(), Error> {
        let mut buf = self.take_buffer()?;
        buf.fill(&params);
        self.send(cmd, buf)
    }

    /// Send a long command (pixels) using 16-bit transfer.
    pub fn send_data(&mut self, cmd: u16, buf: DmaTxBuf) -> Result<(), Error> {
        self.send(cmd, buf)
    }

    fn send<W: Into<u16> + Copy>(&mut self, cmd: W, buf: DmaTxBuf) -> Result<(), Error> {
        self.wait()?;
        let Channel::Bus(bus) = core::mem::take(&mut self.channel) else {
            return Err(Error::BusIsBusy);
        };
        match bus.send(Command::One(cmd), 0, buf) {
            Ok(future) => {
                self.channel = Channel::Future(future);
                Ok(())
            }
            Err((err, bus, buf)) => {
                self.channel = Channel::Bus(bus);
                self.put_buffer(buf)?;
                Err(err.into())
            }
        }
    }
}
