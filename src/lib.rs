#![no_std]

use cortex_m::peripheral;
use embedded_hal::digital::{Error, ErrorKind, ErrorType, OutputPin, StatefulOutputPin};
use rp2040_pac::Peripherals;

// Define custom error enum
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum GpioError {
    GPIO_ERR_INV_PIN,
    GPIO_ERR_INV_MOD,
    GPIO_ERR_NOT_INI,
    GPIO_ERR_UNKNOWN,
}

// Borrow peripherals to write registers
pub struct Gpio {
    peripherals: &Peripherals,
    pin_num: u8,
    initialized: bool,
}

// Self-defined methods
impl Gpio {
    pub fn new(pin_num: u8, peripherals: &Peripherals) -> Self {
        Self {
            pin_num: pin_num,
            peripherals: peripherals,
            initialized: false,
        }
    }
    pub fn init(&mut self) -> Result<(), GpioError> {
        match self.pin_num <= 29 {
            true => {
                self.peripherals
                    .SIO
                    .gpio_oe_set()
                    .write(|w| unsafe { w.bits(1 << self.pin_num) });
                self.peripherals
                    .IO_BANK0
                    .gpio(self.pin_num as usize)
                    .gpio_ctrl()
                    .write(|w| unsafe { w.funcsel().sio().bits(5) });
                self.initialized = true;
                Ok(())
            }
            false => Err(GpioError::GPIO_ERR_INV_PIN),
        }
    }
}

// Define error, for GPIO always default to Other type
impl Error for GpioError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

// Define error type for GPIO as GpioError
impl ErrorType for Gpio {
    type Error = GpioError;
}

impl OutputPin for Gpio {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        // TODO: add checks for output type and if initialized
        if !self.initialized {
            return Err(GpioError::GPIO_ERR_NOT_INI);
        }
        Ok(self
            .peripherals
            .SIO
            .gpio_out_set()
            .write(|w| unsafe { w.bits(0 << self.pin_num) }))
    }
    fn set_high(&mut self) -> Result<(), Self: Error> {
        Ok(peripherals
            .SIO
            .gpio_out_set()
            .write(|w| unsafe { w.bits(1 << self.pin_num) }))
    }
}

impl StatefulOutputPin for Gpio {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.peripherals.SIO.gpio_out().read().bits() >> self.pin_num & 1 == 1)
    }
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.peripherals.SIO.gpio_out().read().bits() >> self.pin_num & 1 == 0)
    }
    fn toggle(&mut self) -> Result<(), Self::Error> {
        Ok(self
            .peripherals
            .SIO
            .gpio_out_xor()
            .write(|w| unsafe { w.bits(1 << 25) }))
    }
}
