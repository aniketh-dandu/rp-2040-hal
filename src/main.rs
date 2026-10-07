#![no_std]
#![no_main]

// Boot2 section
#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// Halts processor on panic
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

    // Clock setup (needs to happen before SPI reset de-assert)
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
    peripherals
        .CLOCKS
        .clk_ref_ctrl()
        .write(|w| w.src().xosc_clksrc());
    peripherals
        .CLOCKS
        .clk_sys_ctrl()
        .write(|w| w.src().clk_ref());
    peripherals
        .CLOCKS
        .clk_peri_ctrl()
        .write(|w| w.auxsrc().clk_sys().enable().set_bit());

    while peripherals.XOSC.status().read().bits() & (1 << 31) == 0 {}

    // De-assert RESETs
    peripherals.RESETS.reset().modify(|_, w| {
        w.io_bank0().clear_bit();
        w.pads_bank0().clear_bit();
        w.spi0().clear_bit()
    });

    // Wait for deassertion to complete
    while peripherals.RESETS.reset_done().read().io_bank0() == false
        || peripherals.RESETS.reset_done().read().pads_bank0() == false
        || peripherals.RESETS.reset_done().read().spi0() == false
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

    // SPI
    // Write 0111 to SPPCR0 DSS for 8-bit data
    // SCR is 11,  CPSDVSR is 2, 12 MHz / (2 * (1 + 11)) = 500 kHz
    peripherals.SPI0.sspcr0().write(|w| unsafe {
        w.dss().bits(7);
        w.scr().bits(11)
    });
    // Enable loopback, is master by default
    peripherals.SPI0.sspcr1().write(|w| {
        w.lbm().set_bit();
        w.sse().set_bit()
    });

    peripherals
        .SPI0
        .sspcpsr()
        .write(|w| unsafe { w.cpsdvsr().bits(2) });
    peripherals
        .SPI0
        .sspdr()
        .write(|w| unsafe { w.data().bits(0x99) });

    while peripherals.SPI0.sspsr().read().rne().bit_is_clear() {}

    let read_value = peripherals.SPI0.sspdr().read().data().bits();
    if read_value == 0x99 {
        _ = led_pin.set_high();
    }
    loop {}
}
