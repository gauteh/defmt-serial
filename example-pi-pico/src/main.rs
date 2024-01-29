//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::{entry};
use defmt;
use defmt_serial as _;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

static SERIAL: StaticCell<bsp::hal::uart::UartPeripheral<_, _, _>> = StaticCell::new();

#[entry]
fn main() -> ! {

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Need to perform clock init before using UART or it will freeze.
    let uart = bsp::hal::uart::UartPeripheral::new(
        pac.UART0, (pins.gpio0.into_mode(),
        pins.gpio1.into_mode()),
        &mut pac.RESETS)
        .enable(
            bsp::hal::uart::UartConfig::default(),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    defmt_serial::defmt_serial(SERIAL.init(uart));
    defmt::warn!("Hello!");

    loop {
        delay.delay_ms(2000u32);
        defmt::info!("Loop!");
        for i in 0..10 {
            delay.delay_ms(100u32);
            defmt::debug!("Inner loop: {}", i);
        }
    }
}
// End of file
