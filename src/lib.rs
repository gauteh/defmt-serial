#![no_std]
//! A defmt target for logging messages over a serial interface. The serial interface must
//! implement [`embedded_hal::blocking::serial::Write`].
//!
//! The received defmt-frames can be read using e.g. `socat` and `defmt-print`, so that you can set
//! it up as `cargo run`ner. See the [example-artemis](https://github.com/gauteh/defmt-serial/tree/main/example-artemis) for how to do that.
//!
//! You can also use it to have defmt work on std/hosted OSes, see [example-std](https://github.com/gauteh/defmt-serial/tree/main/example-std).
//!
//! ```no_run
//! #![no_std]
//! #![no_main]
//!
//!
//! use panic_probe as _;
//! use cortex_m::asm;
//! use cortex_m_rt::entry;
//! use ambiq_hal::{self as hal, prelude::*};
//!
//! use static_cell::StaticCell;
//! use defmt;
//! use defmt_serial as _;
//!
//! static SERIAL: StaticCell<hal::uart::Uart0> = StaticCell::new();
//!
//! #[entry]
//! fn main() -> ! {
//!     let mut dp = hal::pac::Peripherals::take().unwrap();
//!     let pins = hal::gpio::Pins::new(dp.GPIO);
//!
//!     // set up serial
//!     let serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
//!     defmt_serial::defmt_serial(SERIAL.init(serial));
//!
//!     defmt::info!("Hello from defmt!");
//!
//!     loop {
//!         asm::wfi();
//!     }
//! }
//! ```

use core::ptr::addr_of_mut;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::global_logger;
use embedded_io::Write;

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();

/// All of this nonsense is to try and erase the Error type of the `embedded_hal::serial::nb::Write` implementor.
pub trait EraseWrite {
    fn write(&mut self, buf: &[u8]);
    fn flush(&mut self);
}

impl<T: Write> EraseWrite for T {
    fn write(&mut self, buf: &[u8]) {
        self.write_all(buf).ok();
    }

    fn flush(&mut self) {
        self.flush().ok();
    }
}

static mut ERASEDWRITE: Option<&mut dyn EraseWrite> = None;

/// Assign a serial peripheral to receive defmt-messages.
///
///
/// ```no_run
///     static SERIAL: StaticCell<hal::uart::Uart0> = StaticCell::new();
///     let serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
///     defmt_serial::defmt_serial(SERIAL.init(serial));
///
///     defmt::info!("Hello from defmt!");
/// ```
///
/// The peripheral should implement the [`embedded_hal::blocking::serial::Write`] trait. If your HAL
/// already has the non-blocking [`Write`](embedded_hal::serial::Write) implemented, it can opt-in
/// to the [default implementation](embedded_hal::blocking::serial::write::Default).
///
/// Will panic if assigned more than once.
pub fn defmt_serial<T: EraseWrite>(serial: &'static mut T) {
    unsafe {
        critical_section::with(|_| {
            assert!(
                ERASEDWRITE.is_none(),
                "Tried to assign serial port when one was already assigned."
            );
            ERASEDWRITE = Some(serial);
        });
    }
}

/// Release the serial port from defmt.
pub fn release() {
    unsafe {
        critical_section::with(|_| {
            if TAKEN.load(Ordering::Relaxed) {
                panic!("defmt logger taken reentrantly"); // I don't think this actually is
                                                          // possible.
            }

            ERASEDWRITE = None;
        });
    }
}

#[global_logger]
struct GlobalSerialLogger;

unsafe impl defmt::Logger for GlobalSerialLogger {
    fn acquire() {
        let restore = unsafe { critical_section::acquire() };

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly");
        }

        TAKEN.store(true, Ordering::Relaxed);

        unsafe {
            CS_RESTORE = restore;
        }

        unsafe { ENCODER.start_frame(write_serial) }
    }

    unsafe fn release() {
        ENCODER.end_frame(write_serial);
        TAKEN.store(false, Ordering::Relaxed);

        let restore = CS_RESTORE;
        critical_section::release(restore);
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, write_serial);
    }

    unsafe fn flush() {
        if let Some(writer) = &mut *addr_of_mut!(ERASEDWRITE) {
            (*writer).flush();
        }
    }
}

/// Write to serial using proxy function. We must ensure this function is not called
/// several times in parallel.
fn write_serial(remaining: &[u8]) {
    unsafe {
        if let Some(writer) = &mut *addr_of_mut!(ERASEDWRITE) {
            (*writer).write(remaining);
        }
    }
}
