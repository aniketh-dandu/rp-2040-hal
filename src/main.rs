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

// entry point after RAM initialization
// NOTE: never ending function, returns !
#[entry]
fn main() -> ! {
    loop {}
}
