#![no_std]
#![no_main]

use panic_probe as _;
use cortex_m_rt::entry;
use ambiq_hal::{self as hal, prelude::*};
use defmt;
use defmt_serial as _;
use static_cell::StaticCell;

static SERIAL: StaticCell<hal::uart::Uart0> = StaticCell::new();

#[entry]
fn main() -> ! {
    let mut dp = hal::pac::Peripherals::take().unwrap();
    let core = hal::pac::CorePeripherals::take().unwrap();

    let mut delay = hal::delay::Delay::new(core.SYST, &mut dp.CLKGEN);

    let pins = hal::gpio::Pins::new(dp.GPIO);
    let mut led = pins.d19.into_push_pull_output();

    // set up serial
    let serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
    defmt_serial::defmt_serial(SERIAL.init(serial));

    defmt::warn!("Hello!");

    loop {
        delay.delay_ms(2000u32);
        defmt::info!("Loop!");
        for i in 0..10 {
            led.toggle().unwrap();
            delay.delay_ms(100u32);
            defmt::debug!("Inner loop: {}", i);
        }
    }
}
