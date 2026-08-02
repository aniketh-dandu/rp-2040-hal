#![no_std]
#![no_main]

// Boot2 section
#[unsafe(link_section = ".boot2")]
#[unsafe(no_mangle)]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// Halts processor on panic
use cortex_m_rt::entry;
use panic_halt as _;
use rp2040_pac;

// entry point after RAM initialization
// NOTE: never ending function, returns !
#[entry]
fn main() -> ! {
    let mut peripherals = unsafe { rp2040_pac::Peripherals::steal() };

    // De-assert RESETs
    peripherals.RESETS.reset().modify(|_, w| {
        w.io_bank0().clear_bit();
        w.pads_bank0().clear_bit()
    });

    // Wait for deassertion to complete
    while peripherals.RESETS.reset_done().read().io_bank0() == false
        || peripherals.RESETS.reset_done().read().pads_bank0() == false
    {}

    // Set the pin function to SIO (Single-Cycle I/O)
    peripherals
        .IO_BANK0
        .gpio(25)
        .gpio_ctrl()
        .write(|w| unsafe { w.funcsel().sio().bits(5) });
    // Enable output on pin
    peripherals
        .SIO
        .gpio_oe_set()
        .write(|w| unsafe { w.bits(1 << 25) });
    // Write output value
    peripherals
        .SIO
        .gpio_out_set()
        .write(|w| unsafe { w.bits(1 << 25) });
    loop {}
}
