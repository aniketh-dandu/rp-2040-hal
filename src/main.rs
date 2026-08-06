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
use rp2040_pac::{self};

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

    let mut timer_high: u32;
    let mut next_high: u32;
    let mut timer_low: u32;
    let mut prev_time: u64 = 0;
    let mut time: u64;

    loop {
        timer_high = peripherals.TIMER.timerawh().read().bits();
        timer_low = peripherals.TIMER.timerawl().read().bits();
        next_high = peripherals.TIMER.timerawh().read().bits();

        while timer_high != next_high {
            timer_high = next_high;
            timer_low = peripherals.TIMER.timerawl().read().bits();
            next_high = peripherals.TIMER.timerawh().read().bits();
        }

        time = ((timer_high as u64) << 32) | (timer_low as u64);

        if (time - prev_time) > 250000 {
            prev_time = time;
            peripherals
                .SIO
                .gpio_out_xor()
                .write(|w| unsafe { w.bits(1 << 25) });
        }
    }
}
