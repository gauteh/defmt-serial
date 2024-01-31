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

// A mutable reference to the serial is kept in `DefmtSerialHandle` so that it cannot be used at
// the same time as defmt is using it (and thus preventing it from messing up the defmt stream).
//
// defmt also needs a globally accessible reference, so the serial is aliased into this pointer.
static mut ERASEDWRITE: Option<*mut dyn EraseWrite> = None;

#[must_use = "You must hold on to this handle, otherwise the serial port will be de-registered, e.g.: `let _ds = defmt_serial(...)`"]
pub struct DefmtSerialHandle<'serial, T: embedded_hal::blocking::serial::Write<u8, Error = E>, E> {
    #[allow(unused)]
    serial: &'serial mut T, // aliased to ERASEDWRITE for defmt.
}

impl<'serial, T: embedded_hal::blocking::serial::Write<u8, Error = E>, E>
    DefmtSerialHandle<'serial, T, E>
{
    fn new(serial: &'serial mut T) -> DefmtSerialHandle<'serial, T, E> {
        critical_section::with(|_cs| {
            unsafe {
                let serial = serial as &mut dyn EraseWrite;

                // Safety:
                //
                // err..: the pointer in ERASEDWRITE is cleared on DefmtSerialHandle drop.
                // DefmtSerialHandle has the same liftetime as `serial` and holds the reference.
                //
                // so the reference is cleaned up by drop, and the global static is cleared at the
                // same time. it is never accessed by the DefmtSerialHandle. and ERASEDWRITE is
                // only accessed protected by a critical-section.
                ERASEDWRITE = Some(core::mem::transmute(serial as *mut _));
            }
        });

        DefmtSerialHandle { serial }
    }

    /// Borrow the serial. This is unsafe because the serial port is used by defmt, and you may
    /// mess up the defmt stream. Secondly, the serial port may be accessed concurrently by defmt
    /// if you have multiple threads so make sure to guard against that using a critical section or
    /// similar.
    ///
    /// It is better to drop the `DefmtSerialHandle` and re-construct it afterwards.
    pub unsafe fn serial(&mut self) -> &mut T {
        self.serial
    }
}

impl<'serial, T, E> Drop for DefmtSerialHandle<'serial, T, E>
where
    T: embedded_hal::blocking::serial::Write<u8, Error = E>,
{
    fn drop(&mut self) {
        critical_section::with(|_cs| unsafe {
            ERASEDWRITE = None;
        });
    }
}

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
pub fn defmt_serial<'serial, T: embedded_hal::blocking::serial::Write<u8, Error = E>, E>(
    serial: &'serial mut T,
) -> DefmtSerialHandle<T, E> {
    DefmtSerialHandle::new(serial)
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
        if let Some(writer) = ERASEDWRITE {
            let writer = &mut *writer;
            writer.flush();
        }
    }
}

/// Write to serial using proxy function. We must ensure this function is not called
/// several times in parallel.
fn write_serial(remaining: &[u8]) {
    unsafe {
        if let Some(writer) = ERASEDWRITE {
            let writer = &mut *writer;
            writer.write(remaining);
        }
    }
}
