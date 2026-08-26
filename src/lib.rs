#![no_std]

use core::marker::PhantomData;

use embedded_hal::digital::{Error, ErrorKind, ErrorType, OutputPin, StatefulOutputPin};
use rp2040_pac::Peripherals;
use rp2040_pac::{IO_BANK0, SIO};

// Define custom error enum
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum GpioError {
    GPIO_ERR_INV_PIN,
    GPIO_ERR_INV_MOD,
    GPIO_ERR_NOT_INI,
    GPIO_ERR_UNKNOWN,
}

pub struct Input;
pub struct Output;
pub struct Uninitialized;

pub struct GpioNew<'a> {
    io: &'a rp2040_pac::IO_BANK0,
    sio: &'a rp2040_pac::SIO,
}

impl<'a> GpioNew<'a> {
    pub fn new(peripherals: &'a Peripherals) -> Self {
        GpioNew {
            io: &peripherals.IO_BANK0,
            sio: &peripherals.SIO,
        }
    }

    pub fn split(self) -> Pins<'a> {
        Pins {
            gpio0: Pin {
                _mode: PhantomData::<Uninitialized>,
                pin_num: 25,
                sio: &self.sio,
                io: &self.io,
            },
            gpio1: Pin {
                _mode: PhantomData::<Uninitialized>,
                pin_num: 1,
                sio: &self.sio,
                io: &self.io,
            },
        }
    }
}

pub struct Pin<'a, MODE> {
    _mode: PhantomData<MODE>,
    pin_num: u8,
    sio: &'a SIO,
    io: &'a IO_BANK0,
}

pub struct Pins<'a> {
    pub gpio0: Pin<'a, Uninitialized>,
    pub gpio1: Pin<'a, Uninitialized>,
}

// TODO: fix initialization
// TODO: see if there is a better way to handle lifetimes
// TODO: implement input pins

impl<'a> Pin<'a, Uninitialized> {
    pub fn into_output(self) -> Pin<'a, Output> {
        let pin: Pin<Output> = Pin {
            _mode: PhantomData::<Output>,
            pin_num: self.pin_num,
            sio: self.sio,
            io: self.io,
        };
        pin.sio
            .gpio_oe_set()
            .write(|w| unsafe { w.bits(1 << pin.pin_num) });
        pin.io
            .gpio(self.pin_num as usize)
            .gpio_ctrl()
            .write(|w| unsafe { w.funcsel().sio().bits(5) });
        pin
    }
}

impl<'a> OutputPin for Pin<'a, Output> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self
            .sio
            .gpio_out_set()
            .write(|w| unsafe { w.bits(1 << self.pin_num) }))
    }
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self
            .sio
            .gpio_out_clr()
            .write(|w| unsafe { w.bits(1 << self.pin_num) }))
    }
    // fn set_state(&mut self, state: embedded_hal::digital::PinState) -> Result<(), Self::Error> {}
}

impl<'a> StatefulOutputPin for Pin<'a, Output> {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.sio.gpio_out().read().bits() >> self.pin_num & 1 == 1)
    }
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.sio.gpio_out().read().bits() >> self.pin_num & 1 == 0)
    }
    fn toggle(&mut self) -> Result<(), Self::Error> {
        Ok(self
            .sio
            .gpio_out_xor()
            .write(|w| unsafe { w.bits(1 << self.pin_num) }))
    }
}

// Define error, for GPIO always default to Other type
impl Error for GpioError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

// Define error type for GPIO as GpioError
impl<'a> ErrorType for Pin<'a, Output> {
    type Error = GpioError;
}

/*
// Borrow peripherals to write registers
// Add lifetime for borrow
pub struct Gpio<'a> {
    peripherals: &'a Peripherals,
    pin_num: u8,
    initialized: bool,
}

// Self-defined methods
impl<'a> Gpio<'a> {
    pub fn new(pin_num: u8, peripherals: &'a Peripherals) -> Self {
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

impl OutputPin for Gpio<'_> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(GpioError::GPIO_ERR_NOT_INI);
        }
        Ok(self
            .peripherals
            .SIO
            .gpio_out_clr()
            .write(|w| unsafe { w.bits(1 << self.pin_num) }))
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(GpioError::GPIO_ERR_NOT_INI);
        }
    }
}

impl StatefulOutputPin for Gpio<'_> {
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
            .write(|w| unsafe { w.bits(1 << self.pin_num) }))
    }
}
*/
