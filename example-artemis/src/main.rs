#![no_std]
#![no_main]

use panic_probe as _;
use cortex_m::asm;
use cortex_m_rt::entry;
use ambiq_hal as hal;
use defmt;
use defmt_rtt as _;

#[entry]
fn main() -> ! {
    defmt::info!("Hello!");
    loop {
        asm::nop();
    }
}
