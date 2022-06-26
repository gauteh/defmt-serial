#![no_std]
#![no_main]

use panic_probe as _;
use cortex_m::asm;
use cortex_m_rt::entry;
use ambiq_hal::{self as hal, prelude::*};
use defmt;
use defmt_serial as _;

#[entry]
fn main() -> ! {
    let mut dp = hal::pac::Peripherals::take().unwrap();
    let core = hal::pac::CorePeripherals::take().unwrap();

    let mut delay = hal::delay::Delay::new(core.SYST, &mut dp.CLKGEN);

    let pins = hal::gpio::Pins::new(dp.GPIO);
    let mut led = pins.d19.into_push_pull_output();

    // set up serial
    let mut serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
    defmt_serial::defmt_serial!(serial, hal::uart::Uart0);

    defmt::info!("Hello!");

    loop {
        led.toggle().unwrap();
        delay.delay_ms(2000u32);
        asm::nop();
    }
}
