use core::marker::PhantomData;

use embedded_hal::digital::{
    Error, ErrorKind, ErrorType, InputPin, OutputPin, PinState, StatefulOutputPin,
};
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
pub struct I2C;

pub struct Gpio<'a> {
    io: &'a IO_BANK0,
    sio: &'a SIO,
}

impl<'a> Gpio<'a> {
    pub fn new(peripherals: &'a Peripherals) -> Self {
        Gpio {
            io: &peripherals.IO_BANK0,
            sio: &peripherals.SIO,
        }
    }

    pub fn split(self) -> GpioPins<'a> {
        GpioPins {
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
            gpio2: Pin {
                _mode: PhantomData::<Uninitialized>,
                pin_num: 2,
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

pub struct GpioPins<'a> {
    pub gpio0: Pin<'a, Uninitialized>,
    pub gpio1: Pin<'a, Uninitialized>,
    pub gpio2: Pin<'a, Uninitialized>,
}

// TODO: fix initialization
// TODO: setup each peripheral to own respective register banks

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

    pub fn into_input(self) -> Pin<'a, Input> {
        let pin: Pin<Input> = Pin {
            _mode: PhantomData::<Input>,
            pin_num: self.pin_num,
            sio: self.sio,
            io: self.io,
        };
        pin.sio
            .gpio_oe_clr()
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
    fn set_state(&mut self, state: embedded_hal::digital::PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.set_high(),
            PinState::Low => self.set_low(),
        }
    }
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

impl<'a> InputPin for Pin<'a, Input> {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.sio.gpio_in().read().bits() >> self.pin_num & 1 == 1)
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.sio.gpio_in().read().bits() >> self.pin_num & 1 == 0)
    }
}

// Define error, for GPIO always default to Other type
impl Error for GpioError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

// Define error type for GPIO as GpioError
impl<'a, T> ErrorType for Pin<'a, T> {
    type Error = GpioError;
}
