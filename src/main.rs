#![no_std]
#![no_main]

// Boot2 section
#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// Halts processor on panic
use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_hal::digital::{InputPin, OutputPin, PinState};
use panic_halt as _;
use rp2040_pac::{self};
use rp2040hal::gpio::{Gpio, GpioPins, Input, Output, Pin};

// entry point after RAM initialization
// NOTE: never ending function, returns !
#[entry]
fn main() -> ! {
    let peripherals = unsafe { rp2040_pac::Peripherals::steal() };

    // De-assert RESETs
    peripherals.RESETS.reset().modify(|_, w| {
        w.io_bank0().clear_bit();
        w.pads_bank0().clear_bit()
    });

    // Wait for deassertion to complete
    while peripherals.RESETS.reset_done().read().io_bank0() == false
        || peripherals.RESETS.reset_done().read().pads_bank0() == false
    {}

    let gpio: Gpio = Gpio::new(&peripherals);
    let pins: GpioPins = gpio.split();
    let mut led_pin: Pin<Output> = pins.gpio0.into_output();
    let mut out_pin: Pin<Output> = pins.gpio1.into_output();
    let mut in_pin: Pin<Input> = pins.gpio2.into_input();

    _ = out_pin.set_state(PinState::High);

    let mut timer_high: u32;
    let mut next_high: u32;
    let mut timer_low: u32;
    let mut prev_time: u64 = 0;
    let mut time: u64;

    // Startup delay is 12E6*1E-3/256 = 47 = 0x2F
    peripherals
        .XOSC
        .startup()
        .write(|w| unsafe { w.bits(0x2F) });
    // Enable XOSC
    peripherals
        .XOSC
        .ctrl()
        .write(|w| unsafe { w.bits(0xFAB << 12 | 0xAA0) });

    while peripherals.XOSC.status().read().bits() & (1 << 31) == 0 {}

    loop {
        // Reading lower register locks higher register
        // Read HIGH then LOW, if HIGH has changed, read LOW again to lock the HIGH reg
        // Repeat until HIGH matches when read before and after lock
        timer_high = peripherals.TIMER.timerawh().read().bits();
        timer_low = peripherals.TIMER.timerawl().read().bits();
        next_high = peripherals.TIMER.timerawh().read().bits();

        while timer_high != next_high {
            timer_high = next_high;
            timer_low = peripherals.TIMER.timerawl().read().bits();
            next_high = peripherals.TIMER.timerawh().read().bits();
        }

        time = ((timer_high as u64) << 32) | (timer_low as u64);

        // Poll every 5 ms to check Input
        if (time - prev_time) > 5000 {
            prev_time = time;
            if let Ok(true) = in_pin.is_high() {
                let _ = led_pin.set_high();
            } else {
                let _ = led_pin.set_low();
            }
        }
    }
}
