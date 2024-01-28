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
//! use defmt;
//! use defmt_serial as _;
//!
//! #[entry]
//! fn main() -> ! {
//!     let mut dp = hal::pac::Peripherals::take().unwrap();
//!     let pins = hal::gpio::Pins::new(dp.GPIO);
//!
//!     // set up serial
//!     let mut serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
//!     defmt_serial::defmt_serial(serial);
//!
//!     defmt::info!("Hello from defmt!");
//!
//!     loop {
//!         asm::wfi();
//!     }
//! }
//! ```

use core::sync::atomic::{AtomicBool, Ordering};
use defmt::global_logger;

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();

/// All of this nonsense is to try and erase the Error type of the `embedded_hal::serial::nb::Write` implementor.
trait EraseWrite {
    fn write(&mut self, buf: &[u8]);
    fn flush(&mut self);
}

impl<T: embedded_hal::blocking::serial::Write<u8, Error = E>, E> EraseWrite for T {
    fn write(&mut self, buf: &[u8]) {
        for b in buf {
            self.bwrite_all(&b.to_ne_bytes()).ok();
        }
    }

    fn flush(&mut self) {
        self.bflush().ok();
    }
}

static mut ERASEDWRITE: Option<&mut dyn EraseWrite> = None;

/// Assign a serial peripheral to receive defmt-messages.
///
///
/// ```no_run
///     let mut serial = hal::uart::Uart0::new(dp.UART0, pins.tx0, pins.rx0);
///     defmt_serial::defmt_serial(serial);
///
///     defmt::info!("Hello from defmt!");
/// ```
///
/// The peripheral should implement the [`embedded_hal::blocking::serial::Write`] trait. If your HAL
/// already has the non-blocking [`Write`](embedded_hal::serial::Write) implemented, it can opt-in
/// to the [default implementation](embedded_hal::blocking::serial::write::Default).
pub fn defmt_serial<T: embedded_hal::blocking::serial::Write<u8, Error = E>, E>(
    serial: &'static mut T,
) {
    unsafe {
        let token = critical_section::acquire();
        ERASEDWRITE = Some(serial);
        critical_section::release(token);
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
        if let Some(writer) = &mut ERASEDWRITE {
            writer.flush();
        }
    }
}

/// Write to serial using proxy function. We must ensure this function is not called
/// several times in parallel.
fn write_serial(remaining: &[u8]) {
    unsafe {
        if let Some(writer) = &mut ERASEDWRITE {
            writer.write(remaining);
        }
    }
}
