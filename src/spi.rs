use embedded_hal::spi::{Error, ErrorKind, ErrorType, SpiBus};
use rp2040_pac::Peripherals;
use rp2040_pac::{SPI0, SPI1};

pub struct Spi {}

// TODO: Need to have SPI error types
#[derive(Debug)]
pub enum SpiError {}

impl Error for SpiError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}
impl ErrorType for Spi {
    type Error = SpiError;
}

impl SpiBus for Spi {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}
